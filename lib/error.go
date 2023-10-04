package fmm

import "errors"

var (
	ErrInvalidGameDirectory = errors.New("invalid game directory")
	ErrModAlreadyDisabled   = errors.New("mod is already disabled")
	ErrModAlreadyEnabled    = errors.New("mod is already enabled")
	ErrModNotFoundLocal     = errors.New("mod was not found in the local mods directory")
	ErrNoCompatibleRelease  = errors.New("no compatible release was found")
)
