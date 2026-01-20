package main

import (
	"encoding/json"
	"os"
	"runtime"
	"strings"
)

func main() {
	log := newLogger()
	log.Info("Go version: " + runtime.Version())

	config, ok := loadSearchConfigFromEnv()
	if !ok {
		return
	}

	searchGlob := buildSearchPathString(config.SearchPathsValue, config.HomePath)
	log.Info("Search glob: " + searchGlob)

	matches := findDirectories(config.SearchPaths)
	list := buildList(matches, config.Ignore)

	keyword := ""
	if len(os.Args) > 1 {
		keyword = strings.TrimSpace(os.Args[1])
	}

	results := buildResults(list, keyword)
	if keyword != "" {
		log.Info(formatMatchCount(len(results)))
	}

	items := make([]alfredItem, 0, len(results))
	for _, result := range results {
		items = append(items, newAlfredItem(result.Item))
	}

	payload, err := json.Marshal(alfredOutput{Items: items})
	if err != nil {
		log.Info("Failed to encode Alfred output: " + err.Error())
		return
	}
	_, _ = os.Stdout.Write(payload)
}
