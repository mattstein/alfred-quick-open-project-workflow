package main

import (
	"fmt"
	"os"
	"time"
)

type logger struct {
	enabled bool
	prefix  string
}

func newLogger() *logger {
	enabled := os.Getenv("alfred_debug") != ""
	prefix := "alfred"
	if name := os.Getenv("alfred_workflow_name"); name != "" {
		prefix = name
	} else if bundleID := os.Getenv("alfred_workflow_bundleid"); bundleID != "" {
		prefix = bundleID
	}
	return &logger{enabled: enabled, prefix: prefix}
}

func (l *logger) Info(message string) {
	if !l.enabled {
		return
	}
	timestamp := time.Now().Format("2006-01-02 15:04:05")
	fmt.Fprintf(os.Stderr, "[%s] %s %s\n", l.prefix, timestamp, message)
}

func formatMatchCount(count int) string {
	return fmt.Sprintf("Matching results: %d", count)
}
