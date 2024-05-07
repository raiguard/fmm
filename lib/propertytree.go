package fmm

import (
	"errors"
	"fmt"
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

func readPropertyTree(r DatReader) (PropertyTree, error) {
	kind := r.ReadUint8()
	r.ReadBool() // Internal flag that we do not care about
	switch kind {
	case 0:
		return &PropertyTreeNone{}, nil
	case 1:
		return ptr(PropertyTreeBool(r.ReadBool())), nil
	case 2:
		return ptr(PropertyTreeNumber(r.ReadDouble())), nil
	case 3:
		return ptr(PropertyTreeString(r.ReadOptionalString())), nil
	case 4:
		length := r.ReadUint32()
		res := []PropertyTree{}
		for i := uint32(0); i < length; i++ {
			r.ReadOptionalString()
			val, err := readPropertyTree(r)
			if err != nil {
				return nil, err
			}
			res = append(res, val)
		}
		return ptr(PropertyTreeList(res)), nil
	case 5:
		length := r.ReadUint32()
		res := map[string]PropertyTree{}
		for i := uint32(0); i < length; i++ {
			key := r.ReadOptionalString()
			val, err := readPropertyTree(r)
			if err != nil {
				return nil, err
			}
			res[key] = val
		}
		return ptr(PropertyTreeDict(res)), nil
	}

	return nil, errors.New(fmt.Sprintf("Unknown property tree kind: %d", kind))
}
