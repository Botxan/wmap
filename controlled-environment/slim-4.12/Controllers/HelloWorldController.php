<?php declare(strict_types=1);

namespace Controllers;

use Slim\Http\ServerRequest;
use Slim\Http\Response;

class HelloWorldController {
    public function index(ServerRequest $request, Response $response, $args): Response {
        $response->getBody()->write("Hello Wmap!");
        return $response;
    }
}
