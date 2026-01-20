package main

import (
	"os"
	"path/filepath"
	"sort"
	"strings"
)

const (
	defaultIgnorePatterns = ".,..,.DS_Store"
	fuzzyThreshold        = 0.3
)

type listItem struct {
	Folder      string
	FolderLower string
	Path        string
}

type scoredItem struct {
	Item   listItem
	Score  float64
	Index  int
	Exact  bool
	Prefix bool
}

type searchConfig struct {
	SearchPathsValue string
	HomePath         string
	SearchPaths      []string
	Ignore           map[string]struct{}
}

func loadSearchConfigFromEnv() (searchConfig, bool) {
	searchPathsValue := os.Getenv("SEARCH_PATHS")
	if strings.TrimSpace(searchPathsValue) == "" {
		return searchConfig{}, false
	}

	homePath := os.Getenv("HOME")

	ignoreRaw, ok := os.LookupEnv("IGNORE_PATTERNS")
	if !ok {
		ignoreRaw = defaultIgnorePatterns
	}

	ignore := make(map[string]struct{})
	for _, pattern := range splitAndTrim(ignoreRaw) {
		ignore[pattern] = struct{}{}
	}

	return searchConfig{
		SearchPathsValue: searchPathsValue,
		HomePath:         homePath,
		SearchPaths:      parseSearchPaths(searchPathsValue, homePath),
		Ignore:           ignore,
	}, true
}

func buildList(matches []string, ignore map[string]struct{}) []listItem {
	list := make([]listItem, 0, len(matches))
	for _, match := range matches {
		folder := filepath.Base(strings.TrimRight(match, string(os.PathSeparator)))
		if _, blocked := ignore[folder]; blocked {
			continue
		}
		list = append(list, listItem{
			Folder:      folder,
			FolderLower: strings.ToLower(folder),
			Path:        match,
		})
	}
	return list
}

func findDirectories(searchPaths []string) []string {
	var matches []string
	for _, base := range searchPaths {
		bases := []string{base}
		if hasGlobMeta(base) {
			globbed, err := filepath.Glob(base)
			if err != nil || len(globbed) == 0 {
				continue
			}
			bases = globbed
		}

		for _, root := range bases {
			entries, err := os.ReadDir(root)
			if err != nil {
				continue
			}
			for _, entry := range entries {
				path := filepath.Join(root, entry.Name())
				if entry.IsDir() {
					matches = append(matches, path)
					continue
				}
				if entry.Type()&os.ModeSymlink != 0 {
					info, err := entry.Info()
					if err == nil && info.IsDir() {
						matches = append(matches, path)
					}
				}
			}
		}
	}
	sort.Strings(matches)
	return matches
}

func hasGlobMeta(path string) bool {
	return strings.ContainsAny(path, "*?[")
}

func buildResults(list []listItem, keyword string) []scoredItem {
	results := make([]scoredItem, 0, len(list))
	keyword = strings.TrimSpace(keyword)
	if keyword == "" {
		for index, item := range list {
			results = append(results, scoredItem{
				Item:  item,
				Score: 0,
				Index: index,
			})
		}
		return results
	}

	keywordLower := strings.ToLower(keyword)
	for index, item := range list {
		folderLower := item.FolderLower
		if folderLower == "" {
			folderLower = strings.ToLower(item.Folder)
		}
		score, matched := fuzzyMatchScoreLower(keywordLower, folderLower)
		if !matched || score > fuzzyThreshold {
			continue
		}
		exact := folderLower == keywordLower
		results = append(results, scoredItem{
			Item:   item,
			Score:  score,
			Index:  index,
			Exact:  exact,
			Prefix: !exact && strings.HasPrefix(folderLower, keywordLower),
		})
	}

	sort.SliceStable(results, func(i, j int) bool {
		if results[i].Exact != results[j].Exact {
			return results[i].Exact
		}
		if results[i].Prefix != results[j].Prefix {
			return results[i].Prefix
		}
		if results[i].Score == results[j].Score {
			return results[i].Index < results[j].Index
		}
		return results[i].Score < results[j].Score
	})

	return results
}

// fuzzyMatchScoreLower expects lowercase pattern and text for consistent scoring.
func fuzzyMatchScoreLower(pattern, text string) (float64, bool) {
	if pattern == "" {
		return 0, true
	}
	if strings.Contains(text, pattern) {
		return 0, true
	}

	patternRunes := []rune(pattern)
	textRunes := []rune(text)

	patternIndex := 0
	lastMatch := -1
	gap := 0

	for i, r := range textRunes {
		if patternIndex >= len(patternRunes) {
			break
		}
		if r == patternRunes[patternIndex] {
			if lastMatch != -1 {
				gap += i - lastMatch - 1
			}
			lastMatch = i
			patternIndex++
		}
	}

	if patternIndex != len(patternRunes) {
		return 1, false
	}

	maxGap := len(textRunes) - len(patternRunes)
	gapScore := 0.0
	if maxGap > 0 {
		gapScore = float64(gap) / float64(maxGap)
	}

	lengthPenalty := 0.0
	if len(textRunes) > 0 {
		lengthPenalty = float64(len(textRunes)-len(patternRunes)) / float64(len(textRunes))
	}

	score := gapScore*0.7 + lengthPenalty*0.3
	if score < 0 {
		score = 0
	} else if score > 1 {
		score = 1
	}

	return score, true
}
