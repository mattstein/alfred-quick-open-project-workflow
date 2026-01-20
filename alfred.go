package main

const (
	iconTypeFile = "fileicon"
)

const (
	vsCodeApp   = "/Applications/Visual Studio Code.app"
	phpStormApp = "/Applications/PhpStorm.app"
	iTermApp    = "/Applications/iTerm.app"
	finderApp   = "/System/Library/CoreServices/Finder.app"
)

type alfredOutput struct {
	Items []alfredItem `json:"items"`
}

type alfredItem struct {
	Title    string      `json:"title"`
	Subtitle string      `json:"subtitle,omitempty"`
	Arg      string      `json:"arg,omitempty"`
	Icon     *alfredIcon `json:"icon,omitempty"`
	Mods     *alfredMods `json:"mods,omitempty"`
}

type alfredMods struct {
	Cmd   *alfredMod `json:"cmd,omitempty"`
	Shift *alfredMod `json:"shift,omitempty"`
	Ctrl  *alfredMod `json:"ctrl,omitempty"`
}

type alfredMod struct {
	Icon *alfredIcon `json:"icon,omitempty"`
}

type alfredIcon struct {
	Path string `json:"path"`
	Type string `json:"type,omitempty"`
}

func newAlfredItem(item listItem) alfredItem {
	return alfredItem{
		Title:    item.Folder,
		Subtitle: item.Path,
		Arg:      item.Path,
		Icon:     iconForFilePath(vsCodeApp),
		Mods: &alfredMods{
			Cmd:   &alfredMod{Icon: iconForFilePath(phpStormApp)},
			Shift: &alfredMod{Icon: iconForFilePath(iTermApp)},
			Ctrl:  &alfredMod{Icon: iconForFilePath(finderApp)},
		},
	}
}

func iconForFilePath(path string) *alfredIcon {
	return &alfredIcon{
		Path: path,
		Type: iconTypeFile,
	}
}
