<?php

require_once __DIR__.'/../vendor/autoload.php';

$app = new Silex\Application();
// $app['debug'] = true;

/* *** Wmap *** */
$app->get('/hello/index', 'Controllers\HelloWorldController::getIndex');

$app->run();