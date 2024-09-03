#!/bin/sh
# create project
rm -rf _wmap/temp
composer create-project --prefer-dist laravel/lumen:10.0.* ./_wmap/temp --ansi
mv ./_wmap/temp/{.,}* ./

# have the route & controller
yes|cp -rf _wmap/lumen/. ./

# some enhancements
composer install --no-dev -o --ansi
chmod -R o+w storage
rm ./public/.htaccess