package main

import "strings"

func buildSearchPathString(settingValue, homePath string) string {
	paths := parseSearchPaths(settingValue, homePath)
	return "{" + strings.Join(paths, ",") + "}/*"
}

func parseSearchPaths(settingValue, homePath string) []string {
	raw := strings.Split(settingValue, ",")
	paths := make([]string, 0, len(raw))
	for _, path := range raw {
		path = strings.TrimSpace(path)
		path = expandTilde(path, homePath)
		paths = append(paths, path)
	}
	return paths
}

func expandTilde(path, homePath string) string {
	if strings.HasPrefix(path, "~/") {
		return homePath + "/" + strings.TrimPrefix(path, "~/")
	}
	return path
}

func splitAndTrim(value string) []string {
	raw := strings.Split(value, ",")
	out := make([]string, 0, len(raw))
	for _, item := range raw {
		out = append(out, strings.TrimSpace(item))
	}
	return out
}
