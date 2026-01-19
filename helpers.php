<?php

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
