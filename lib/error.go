package fmm

type errInvalidGameDirectory struct{}

func (m *errInvalidGameDirectory) Error() string {
	return "invalid game directory"
}

var (
	ErrInvalidGameDirectory = &errInvalidGameDirectory{}
)
