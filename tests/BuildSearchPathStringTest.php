<?php

require_once __DIR__.'/../helpers.php';

describe('buildSearchPathString', function () {
    it('wraps a single path in braces with glob wildcard', function () {
        $result = buildSearchPathString('/Projects', '/Users/test');

        expect($result)->toBe('{/Projects}/*');
    });

    it('joins multiple comma-separated paths', function () {
        $result = buildSearchPathString('/Projects,/Work', '/Users/test');

        expect($result)->toBe('{/Projects,/Work}/*');
    });

    it('expands tilde to home directory', function () {
        $result = buildSearchPathString('~/Projects', '/Users/test');

        expect($result)->toBe('{/Users/test/Projects}/*');
    });

    it('expands tilde in multiple paths', function () {
        $result = buildSearchPathString('~/Projects,~/Work', '/Users/test');

        expect($result)->toBe('{/Users/test/Projects,/Users/test/Work}/*');
    });

    it('handles mixed absolute and tilde paths', function () {
        $result = buildSearchPathString('~/Projects,/var/www', '/Users/test');

        expect($result)->toBe('{/Users/test/Projects,/var/www}/*');
    });

    it('trims whitespace from paths', function () {
        $result = buildSearchPathString('  /Projects  ,  /Work  ', '/Users/test');

        expect($result)->toBe('{/Projects,/Work}/*');
    });

    it('trims whitespace and expands tilde together', function () {
        $result = buildSearchPathString('  ~/Projects  ', '/Users/test');

        expect($result)->toBe('{/Users/test/Projects}/*');
    });
});
