package fmm

import "fmt"

type (
	errInvalidGameDirectory struct{}
	errModAlreadyDisabled   struct{ ModIdent }
	errModAlreadyEnabled    struct{ ModIdent }
	errModDoesNotExist      struct{ ModIdent }
)

func (m *errInvalidGameDirectory) Error() string {
	return "invalid game directory"
}

func (m *errModAlreadyDisabled) Error() string {
	return fmt.Sprintf("%s is already disabled", m.ToString())
}

func (m *errModAlreadyEnabled) Error() string {
	return fmt.Sprintf("%s is already enabled", m.ToString())
}

func (m *errModDoesNotExist) Error() string {
	return fmt.Sprintf("%s does not exist", m.ToString())
}

var (
	ErrInvalidGameDirectory = &errInvalidGameDirectory{}
	ErrModAlreadyDisabled   = &errModAlreadyDisabled{}
	ErrModAlreadyEnabled    = &errModAlreadyEnabled{}
	ErrModDoesNotExist      = &errModDoesNotExist{}
)
