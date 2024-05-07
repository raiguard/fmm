package fmm

// Function ptr returns a pointer to T.
func ptr[T any](val T) *T {
	return &val
}
