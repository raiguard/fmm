package fmm

import (
	"bufio"
	"encoding/binary"
	"fmt"
	"io"
	"math"
)

type PropertyTree interface {
	ptree()
}

type (
	PropertyTreeNone   struct{}
	PropertyTreeBool   bool
	PropertyTreeNumber float64
	PropertyTreeString string
	PropertyTreeList   []PropertyTree
	PropertyTreeDict   map[string]PropertyTree
)

func (self *PropertyTreeNone) ptree()   {}
func (self *PropertyTreeBool) ptree()   {}
func (self *PropertyTreeNumber) ptree() {}
func (self *PropertyTreeString) ptree() {}
func (self *PropertyTreeList) ptree()   {}
func (self *PropertyTreeDict) ptree()   {}

type DatReader struct {
	reader *bufio.Reader
}

func newDatReader(reader io.Reader) DatReader {
	return DatReader{
		reader: bufio.NewReader(reader),
	}
}

func (r *DatReader) Read(buf []byte) (int, error) {
	return r.reader.Read(buf)
}

func (d *DatReader) ReadBool() bool {
	var value bool
	if err := binary.Read(d, binary.LittleEndian, &value); err != nil {
		panic(err)
	}
	return value
}

func (d *DatReader) ReadUint8() uint8 {
	value, err := d.reader.ReadByte()
	if err != nil {
		panic(err)
	}
	return uint8(value)
}

func (d *DatReader) ReadUint16() uint16 {
	var value uint16
	if err := binary.Read(d, binary.LittleEndian, &value); err != nil {
		panic(err)
	}
	return value
}

func (d *DatReader) ReadUint16Optimized() uint16 {
	first := d.ReadUint8()
	if first < math.MaxUint8 {
		return uint16(first)
	}
	return d.ReadUint16()
}

func (d *DatReader) ReadUint32() uint32 {
	var val uint32
	if err := binary.Read(d, binary.LittleEndian, &val); err != nil {
		panic(err)
	}
	return val
}

func (d *DatReader) ReadUint32Optimized() uint32 {
	first := d.ReadUint8()
	if first < math.MaxUint8 {
		return uint32(first)
	}
	return d.ReadUint32()
}

func (d *DatReader) ReadDouble() float64 {
	var val float64
	if err := binary.Read(d, binary.LittleEndian, &val); err != nil {
		panic(err)
	}
	return val
}

func (d *DatReader) ReadString() string {
	length := d.ReadUint32Optimized()
	stringBuf := make([]byte, length)
	io.ReadFull(d.reader, stringBuf)
	return string(stringBuf)
}

func (d *DatReader) ReadStringOptional() string {
	empty := d.ReadBool()
	if empty {
		return ""
	}
	return d.ReadString()
}

func (d *DatReader) ReadVersionOptimized(withBuild bool) Version {
	ver := Version{
		d.ReadUint16Optimized(),
		d.ReadUint16Optimized(),
		d.ReadUint16Optimized(),
	}
	if withBuild {
		ver[3] = d.ReadUint16Optimized()
	}
	return ver
}

func (d *DatReader) ReadVersionUnoptimized() Version {
	return Version{
		d.ReadUint16(),
		d.ReadUint16(),
		d.ReadUint16(),
		d.ReadUint16(),
	}
}

func (d *DatReader) ReadModWithCRC() ModIdent {
	name := d.ReadString()
	version := d.ReadVersionOptimized(false)
	d.ReadUint32() // CRC
	return ModIdent{name, &version}
}

func (d *DatReader) ReadPropertyTree() PropertyTree {
	kind := d.ReadUint8()
	d.ReadBool() // Internal flag that we do not care about
	switch kind {
	case 0:
		return &PropertyTreeNone{}
	case 1:
		return ptr(PropertyTreeBool(d.ReadBool()))
	case 2:
		return ptr(PropertyTreeNumber(d.ReadDouble()))
	case 3:
		return ptr(PropertyTreeString(d.ReadStringOptional()))
	case 4:
		length := d.ReadUint32()
		res := []PropertyTree{}
		for i := uint32(0); i < length; i++ {
			d.ReadStringOptional()
			res = append(res, d.ReadPropertyTree())
		}
		return ptr(PropertyTreeList(res))
	case 5:
		length := d.ReadUint32()
		res := map[string]PropertyTree{}
		for i := uint32(0); i < length; i++ {
			res[d.ReadStringOptional()] = d.ReadPropertyTree()
		}
		return ptr(PropertyTreeDict(res))
	}

	panic(fmt.Sprintf("unknown property tree kind: %d\n", kind))
}

type DatWriter struct {
	writer *bufio.Writer
}

func (w *DatWriter) Write(buf []byte) (int, error) {
	return w.writer.Write(buf)
}

func newDatWriter(writer io.Writer) DatWriter {
	return DatWriter{
		writer: bufio.NewWriter(writer),
	}
}

func (w *DatWriter) WriteBool(value bool) {
	binary.Write(w, binary.LittleEndian, value)
}

func (w *DatWriter) WriteUint8(value uint8) {
	binary.Write(w, binary.LittleEndian, value)
}

func (w *DatWriter) WriteUint16(value uint16) {
	binary.Write(w, binary.LittleEndian, value)
}

func (w *DatWriter) WriteUint16Optimized(value uint16) {
	if value < math.MaxUint8 {
		w.WriteUint8(uint8(value))
	} else {
		w.WriteUint16(value)
	}
}

func (w *DatWriter) WriteUint32(value uint32) {
	binary.Write(w, binary.LittleEndian, value)
}

func (w *DatWriter) WriteUint32Optimized(value uint32) {
	if value < math.MaxUint8 {
		w.WriteUint8(uint8(value))
	} else {
		w.WriteUint32(value)
	}
}

func (w *DatWriter) WriteDouble(value float64) {
	binary.Write(w, binary.LittleEndian, value)
}

func (w *DatWriter) WriteString(value string) {
	length := len(value)
	if length > math.MaxUint32 {
		panic("PropertyTree string is too long")
	}
	w.WriteUint32Optimized(uint32(length))
	_, err := w.writer.WriteString(value)
	if err != nil {
		panic(err)
	}
}

func (w *DatWriter) WriteStringOptional(value string) {
	if value == "" {
		w.WriteBool(true)
		return
	}
	w.WriteBool(false)
	w.WriteString(value)
}

func (w *DatWriter) WriteVersionUnoptimized(version Version) {
	w.WriteUint16(version[0])
	w.WriteUint16(version[1])
	w.WriteUint16(version[2])
	w.WriteUint16(version[3])
}

func (w *DatWriter) WritePropertyTree(pt PropertyTree) {
	switch val := pt.(type) {
	case *PropertyTreeNone:
		w.WriteUint8(0) // Type
		w.WriteUint8(0) // Unused internal flag
	case *PropertyTreeBool:
		w.WriteUint8(1) // Type
		w.WriteUint8(0) // Unused internal flag
		w.WriteBool(bool(*val))
	case *PropertyTreeNumber:
		w.WriteUint8(2) // Type
		w.WriteUint8(0) // Unused internal flag
		w.WriteDouble(float64(*val))
	case *PropertyTreeString:
		w.WriteUint8(3) // Type
		w.WriteUint8(0) // Unused internal flag
		w.WriteStringOptional(string(*val))
	case *PropertyTreeList:
		w.WriteUint8(4) // Type
		w.WriteUint8(0) // Unused internal flag
		list := []PropertyTree(*val)
		w.WriteUint32(uint32(len(list)))
		for _, child := range list {
			w.WriteStringOptional("")
			w.WritePropertyTree(child)
		}
	case *PropertyTreeDict:
		w.WriteUint8(5) // Type
		w.WriteUint8(0) // Unused internal flag
		dict := map[string]PropertyTree(*val)
		w.WriteUint32(uint32(len(dict)))
		for key, child := range dict {
			w.WriteStringOptional(key)
			w.WritePropertyTree(child)
		}
	}
}
