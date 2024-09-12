<?php

include 'vendor/autoload.php';

use Alfred\Workflows\Workflow;
use Alfred\Workflows\ParamBuilder\Mod;

$workflow = new Workflow();

if (empty($workflow->env('SEARCH_PATHS'))) {
    return;
}

// Expand ignored file setting into an array
$ignore = array_map(
    'trim',
    explode(',', $workflow->env('IGNORE_PATTERNS', '.,..,.DS_Store'))
);

// Normalize comma-separated setting value into a glob-friendly pattern
$searchPathString = buildSearchPathString(
    $workflow->env('SEARCH_PATHS'),
    $workflow->env('HOME'),
);

// Match directories
$matches = glob($searchPathString, GLOB_ONLYDIR | GLOB_BRACE);

// Build a keyed list of directory folder names and full paths
$list = [];

foreach ($matches as $match) {
    $folder = basename($match);
    if (!in_array($folder, $ignore, true)) {
        $list[] = [
            'folder' => $folder,
            'path' => $match,
        ];
    }
}

// Prepare Fuse to search folder names and be slightly picky about it
$fuse = new \Fuse\Fuse($list, [
    'keys' => ['folder'],
    'minMatchCharLength' => 2,
    'threshold' => 0.3,
]);

// Rock and roll
$results = $fuse->search($workflow->argument());

foreach ($results as $result) {
    $workflow->item()
        ->title($result['item']['folder'])
        ->subtitle($result['item']['path'])
        ->arg($result['item']['path'])
        ->iconForFilePath('/Applications/Visual Studio Code.app')
        ->mod(Mod::cmd()->iconForFilePath('/Applications/PhpStorm.app'))
        ->mod(Mod::shift()->iconForFilePath('/Applications/iTerm.app'))
        ->mod(Mod::ctrl()->iconForFilePath('/System/Library/CoreServices/Finder.app'));
}

$workflow->output();

/**
 * Takes the comma-separated setting value of paths, cleans any leading
 * or trailing space, expands relative home references (`~/`) into
 * absolute paths, and joins them back together into a glob search string
 * using braces (`{/path/one,/path/two}/*`).
 *
 * @param string $settingValue
 * @param string $homePath
 * @return string
 */
function buildSearchPathString(string $settingValue, string $homePath): string
{
    $searchPaths = array_map(static function($path) use ($homePath) {
        $path = trim($path);
        if (str_starts_with($path, '~/')) {
            // Expand `~/` to full path
            $fullHomePath = $homePath.'/';
            return substr_replace(
                $path,
                $fullHomePath,
                0,
                strlen('~/')
            );
        }
        return $path;
    }, explode(',', $settingValue));

    return '{'.implode(',', $searchPaths).'}/*';
}
