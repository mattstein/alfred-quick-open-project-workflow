package main

import "testing"

func TestExactMatchSortsFirst(t *testing.T) {
	list := []listItem{
		{Folder: "alpha-beta", Path: "/tmp/alpha-beta"},
		{Folder: "alpha", Path: "/tmp/alpha"},
	}

	results := buildResults(list, "alpha")
	if len(results) != 2 {
		t.Fatalf("expected 2 results, got %d", len(results))
	}
	if results[0].Item.Folder != "alpha" {
		t.Fatalf("expected exact match first, got %q", results[0].Item.Folder)
	}
}

func TestPrefixMatchSortsAheadOfContains(t *testing.T) {
	list := []listItem{
		{Folder: "beta-alpha", Path: "/tmp/beta-alpha"},
		{Folder: "alpha-beta", Path: "/tmp/alpha-beta"},
		{Folder: "alpha", Path: "/tmp/alpha"},
	}

	results := buildResults(list, "alpha")
	if len(results) != 3 {
		t.Fatalf("expected 3 results, got %d", len(results))
	}
	if results[0].Item.Folder != "alpha" {
		t.Fatalf("expected exact match first, got %q", results[0].Item.Folder)
	}
	if results[1].Item.Folder != "alpha-beta" {
		t.Fatalf("expected prefix match second, got %q", results[1].Item.Folder)
	}
	if results[2].Item.Folder != "beta-alpha" {
		t.Fatalf("expected contains match last, got %q", results[2].Item.Folder)
	}
}
