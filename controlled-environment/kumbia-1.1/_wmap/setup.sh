#!/bin/sh
# create project
rm -rf _wmap/temp
composer create-project --prefer-dist kumbia/framework:1.1.* ./_wmap/temp --ansi
mv ./_wmap/temp/{.,}* ./

# have the route & controller
yes|cp -r _wmap/kumbia/* ./

# some enhancements
composer install --no-dev --optimize-autoloader --ansi

find . -name \*.htaccess -type f -delete