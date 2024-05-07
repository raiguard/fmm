package fmm

import (
	"bytes"
	"encoding/binary"
	"io"
	"math"
)

type DatReader struct {
	reader *bytes.Reader
}

func newDatReader(source []byte) DatReader {
	return DatReader{
		reader: bytes.NewReader(source),
	}
}

func (d *DatReader) Advance(offset int64) {
	d.reader.Seek(offset, io.SeekCurrent)
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

func (d *DatReader) ReadUint32() uint32 {
	buf := make([]byte, 4)
	io.ReadFull(d.reader, buf)
	return binary.LittleEndian.Uint32(buf)
}

func (d *DatReader) ReadUint16Optimized() uint16 {
	first, _ := d.reader.ReadByte()
	if first < 255 {
		return uint16(first)
	}
	return d.ReadUint16()
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
