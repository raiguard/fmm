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

func (d *DatReader) ReadBool() bool {
	num, _ := d.reader.ReadByte()
	return num == 1
}

func (d *DatReader) ReadModWithCRC() ModIdent {
	name := d.ReadString()
	version := d.ReadOptimizedVersion(false)
	d.ReadUint32() // CRC
	return ModIdent{name, &version}
}

func (d *DatReader) ReadString() string {
	length := d.ReadUint16Optimized()
	stringBuf := make([]byte, length)
	io.ReadFull(d.reader, stringBuf)
	return string(stringBuf)
}

func (d *DatReader) ReadOptionalString() string {
	empty := d.ReadBool()
	if empty {
		return ""
	}
	return d.ReadString()
}

func (d *DatReader) ReadUint8() uint8 {
	byte, _ := d.reader.ReadByte()
	return uint8(byte)
}

func (d *DatReader) ReadUint16() uint16 {
	buf := make([]byte, 2)
	io.ReadFull(d.reader, buf)
	return binary.LittleEndian.Uint16(buf)
}

func (d *DatReader) ReadUint16Optimized() uint16 {
	first, _ := d.reader.ReadByte()
	if first < 255 {
		return uint16(first)
	}
	return d.ReadUint16()
}

func (d *DatReader) ReadUint32() uint32 {
	buf := make([]byte, 4)
	io.ReadFull(d.reader, buf)
	return binary.LittleEndian.Uint32(buf)
}

func (d *DatReader) ReadDouble() float64 {
	buf := make([]byte, 8)
	io.ReadFull(d.reader, buf)
	bits := binary.LittleEndian.Uint64(buf)
	return math.Float64frombits(bits)
}

func (d *DatReader) ReadOptimizedVersion(withBuild bool) Version {
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

func (d *DatReader) ReadUnoptimizedVersion() Version {
	return Version{
		d.ReadUint16(),
		d.ReadUint16(),
		d.ReadUint16(),
		d.ReadUint16(),
	}
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
		return ptr(PropertyTreeString(d.ReadOptionalString()))
	case 4:
		length := d.ReadUint32()
		res := []PropertyTree{}
		for i := uint32(0); i < length; i++ {
			d.ReadOptionalString()
			res = append(res, d.ReadPropertyTree())
		}
		return ptr(PropertyTreeList(res))
	case 5:
		length := d.ReadUint32()
		res := map[string]PropertyTree{}
		for i := uint32(0); i < length; i++ {
			res[d.ReadOptionalString()] = d.ReadPropertyTree()
		}
		return ptr(PropertyTreeDict(res))
	}

	fmt.Printf("Unknown property tree kind: %d\n", kind)
	return nil
}
