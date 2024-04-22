#!/bin/sh
# create project
rm -rf _wmap/temp
composer create-project --prefer-dist cakephp/app:5.0.* ./_wmap/temp --ansi
yes|mv ./_wmap/temp/{.,}* ./

# have the route & controller
yes|cp -r _wmap/cakephp/* ./

# some enhancements
composer dump-autoload -o
composer install --no-interaction --no-dev -o --ansi
rm ./webroot/.htaccess