#!/bin/sh
# create project
rm -rf _wmap/temp
composer create-project --prefer-dist laravel/laravel:10.2.* ./_wmap/temp --ansi
mv ./_wmap/temp/{.,}* ./

# have the route & controller
yes|cp -rf _wmap/laravel/. ./

# some enhancements
composer install --optimize-autoloader --no-dev --ansi
chmod -R o+w storage

rm ./public/.htaccess