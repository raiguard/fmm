package main

import (
	"bytes"
	"encoding/binary"
	"io"
)

type DatReader struct {
	reader     *bytes.Reader
	workingBuf []byte
}

func newDatReader(source []byte) DatReader {
	return DatReader{
		reader:     bytes.NewReader(source),
		workingBuf: make([]byte, 2),
	}
}

func (d *DatReader) Advance(offset int64) {
	d.reader.Seek(offset, io.SeekCurrent)
}

func (d *DatReader) ReadString() string {
	length, _ := d.reader.ReadByte()
	stringBuf := make([]byte, length)
	io.ReadFull(d.reader, stringBuf)
	return string(stringBuf)
}

func (d *DatReader) ReadUint8() uint8 {
	byte, _ := d.reader.ReadByte()
	return uint8(byte)
}

func (d *DatReader) ReadUint16() uint16 {
	io.ReadFull(d.reader, d.workingBuf)
	return binary.LittleEndian.Uint16(d.workingBuf)
}

func (d *DatReader) ReadUnoptimizedVersion() Version {
	return Version{
		d.ReadUint16(),
		d.ReadUint16(),
		d.ReadUint16(),
		d.ReadUint16(),
	}
}
