FROM ubuntu
MAINTAINER dante-signal31 (dante.signal31@gmail.com)
# Abort on error.
RUN set -e
# Install steganer.
RUN apt-get update && \
    # Make sure man is installed to test app manpage.
    apt-get install man -y && \
    # We are going to need wget installed to download test files.
    apt-get install wget -y
RUN apt-get install apt-utils apt-transport-https gnupg ca-certificates -y && \
    echo "deb [trusted=yes] https://dl.bintray.com/dante-signal31/deb generic main" | tee -a /etc/apt/sources.list && \
    apt-key adv --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys 379CE192D401AB61
RUN apt-get update && \
    apt-get install steganer -y
# Prepare test bed.
RUN wget https://homepages.cae.wisc.edu/~ece533/images/lena.png && \
    wget https://homepages.cae.wisc.edu/~ece533/images/lena.bmp && \
    wget https://homepages.cae.wisc.edu/~ece533/images/lena.ppm && \
    wget https://www.gutenberg.org/files/74/74-0.txt

