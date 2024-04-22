#!/bin/sh
# create project
rm -rf _wmap/temp
composer create-project fuel/fuel:^1.9.0 --prefer-dist ./_wmap/temp --ansi
mv ./_wmap/temp/{.,}* ./

# have the route & controller
yes|cp -r _wmap/fuel/* ./

# some enhancements
composer config allow-plugins.composer/installers true
composer install --no-dev -o --ansi
rm ./public/.htaccess