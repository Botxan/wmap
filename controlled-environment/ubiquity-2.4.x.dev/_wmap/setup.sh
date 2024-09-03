#!/bin/sh
# create project
rm -rf _wmap/temp
composer create-project phpmv/ubiquity-project:2.4.x-dev ./_wmap/temp --ansi
mv ./_wmap/temp/{.,}* ./

# have the route & controller
yes|cp -rf _wmap/ubiquity/. ./

# some enhancements
composer install --no-dev --optimize-autoloader --ansi
rm ./public/.htaccess