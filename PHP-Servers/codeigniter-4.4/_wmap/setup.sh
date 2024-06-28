#!/bin/sh
# create project
rm -rf _wmap/temp
composer create-project codeigniter4/appstarter:^4.4 --ansi --no-dev ./_wmap/temp
mv ./_wmap/temp/{.,}* ./

# have the route & controller
yes|cp -r _wmap/codeigniter/* ./

# some enhancements
composer install --no-dev -o
chmod -R o+w writable
rm ./public/.htaccess