<?php declare(strict_types=1);

namespace app\controllers;

use yii\web\Controller;

class HelloworldController extends Controller {
    public function actionIndex() {
        return 'Hello Wmap!';
    }
}