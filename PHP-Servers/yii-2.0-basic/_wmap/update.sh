#!/bin/sh
composer update

# have the route & controller
yes|cp -r _wmap/yii2/* ./

# some enhancements
composer install --no-dev -o
rm ./public/.htaccess