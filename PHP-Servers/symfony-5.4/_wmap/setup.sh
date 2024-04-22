#!/bin/sh
# create project
rm -rf _wmap/temp
composer create-project symfony/skeleton:5.4.* ./_wmap/temp --ansi
mv ./_wmap/temp/{.,}* ./

# have the route & controller
yes|cp -r _wmap/symfony/* ./

# some enhancements
composer dump-env prod --ansi
APP_ENV=prod APP_DEBUG=0 bin/console cache:clear
composer install --no-dev --optimize-autoloader --ansi
chmod -R o+w var