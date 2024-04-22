<?php declare(strict_types=1);

namespace App\Controller;

// such simple controller
class HelloWorldController extends AppController {

    public function display()
    {
        return $this->response->withStringBody('Hello Wmap!');
    }
}
