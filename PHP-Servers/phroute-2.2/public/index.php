<?php

require __DIR__.'/../vendor/autoload.php';

use Phroute\Phroute\RouteCollector;

$router = new RouteCollector();

/* *** Wmap *** */
$router->get('/index.php/hello/index', ['Controllers\HelloWorldController', 'getIndex']);

$uri = parse_url($_SERVER['REQUEST_URI'], PHP_URL_PATH);

// just same as fastroute:
// https://github.com/nikic/FastRoute/issues/110#issuecomment-273760186
// Strip prefix
$prefix = '/wmap/phroute-2.2/public';
if ($prefix !== '' && strpos($uri, $prefix) === 0) {
    $uri = substr($uri, strlen($prefix));
}

# NB. You can cache the return value from $router->getData() so you don't have to create the routes each request - massive speed gains
$dispatcher = new Phroute\Phroute\Dispatcher($router->getData());

$response = $dispatcher->dispatch($_SERVER['REQUEST_METHOD'], $uri);
    
// Print out the value returned from the dispatched function
echo $response;