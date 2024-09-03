<?php
use Slim\Factory\AppFactory;

require __DIR__.'/../vendor/autoload.php';

// Instantiate App
$app = AppFactory::create();

// xampp
$app->setBasePath("/wmap/slim-4.12/public/index.php");

// Add error middleware
$app->addErrorMiddleware(false, true, true);

/* *** Wmap *** */
$app->get('/hello/index', Controllers\HelloWorldController::class . ':index');

$app->run();