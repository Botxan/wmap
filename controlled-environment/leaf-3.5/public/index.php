<?php

require '../vendor/autoload.php';

app()->get("/index.php/hello/index", 'Controllers\HelloWorldController@index');

app()->run();