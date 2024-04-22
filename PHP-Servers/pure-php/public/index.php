<?php

$uri = $_SERVER['REQUEST_URI'];

$prefix = '/wmap/pure-php/public';

require_once ('../Controllers/HelloWorldController.php');

// Strip query string (?foo=bar) and decode URI
if (false !== $pos = strpos($uri, '?')) {
    $uri = substr($uri, 0, $pos);
}
$uri = rawurldecode($uri);

if ($prefix !== '' && strpos($uri, $prefix) === 0) {
    $uri = substr($uri, strlen($prefix));
}

switch ($uri) {
    case '/index.php/hello/index':
    call_user_func ([new Controllers\HelloWorldController, 'getIndex']);
    break;
    default:
    http_response_code(404);
    print 'Error 404';
    break;
}