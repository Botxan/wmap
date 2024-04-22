#!/bin/sh
composer update

# have the route & controller
yes|cp -r _wmap/cakephp/* ./

# some enhancements
composer dump-autoload -o
composer install --no-interaction --no-dev -o
rm ./webroot/.htaccess