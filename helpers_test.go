package main

import "testing"

func TestBuildSearchPathString(t *testing.T) {
	tests := []struct {
		name     string
		paths    string
		home     string
		expected string
	}{
		{
			name:     "single path",
			paths:    "/Projects",
			home:     "/Users/test",
			expected: "{/Projects}/*",
		},
		{
			name:     "multiple paths",
			paths:    "/Projects,/Work",
			home:     "/Users/test",
			expected: "{/Projects,/Work}/*",
		},
		{
			name:     "tilde expansion",
			paths:    "~/Projects",
			home:     "/Users/test",
			expected: "{/Users/test/Projects}/*",
		},
		{
			name:     "tilde expansion in multiple paths",
			paths:    "~/Projects,~/Work",
			home:     "/Users/test",
			expected: "{/Users/test/Projects,/Users/test/Work}/*",
		},
		{
			name:     "mixed absolute and tilde paths",
			paths:    "~/Projects,/var/www",
			home:     "/Users/test",
			expected: "{/Users/test/Projects,/var/www}/*",
		},
		{
			name:     "trims whitespace",
			paths:    "  /Projects  ,  /Work  ",
			home:     "/Users/test",
			expected: "{/Projects,/Work}/*",
		},
		{
			name:     "trims whitespace and expands tilde",
			paths:    "  ~/Projects  ",
			home:     "/Users/test",
			expected: "{/Users/test/Projects}/*",
		},
	}

	for _, test := range tests {
		test := test
		t.Run(test.name, func(t *testing.T) {
			result := buildSearchPathString(test.paths, test.home)
			if result != test.expected {
				t.Fatalf("expected %q, got %q", test.expected, result)
			}
		})
	}
}
