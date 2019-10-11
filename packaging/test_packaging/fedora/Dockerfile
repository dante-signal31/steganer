FROM fedora
MAINTAINER dante-signal31 (dante.signal31@gmail.com)
# Abort on error.
RUN set -e
# Install steganer.
RUN yum update -y && \
    # Make sure man is installed to test app manpage.
    yum install man -y && \
    # We are going to need wget installed to download test files.
    yum install wget -y
RUN wget https://bintray.com/dante-signal31/rpm/rpm -O bintray-dante-signal31-rpm.repo && \
    mv bintray-dante-signal31-rpm.repo /etc/yum.repos.d/ && \
    yum update -y
# Fedora docker image is configured to not include manpages from installed packages to save space.
# I remove that restriction to allow steganer manpage to be properly installed.
RUN sed -i '/tsflags=nodocs/d' /etc/dnf/dnf.conf
#RUN yum install steganer -y
# Prepare test bed.
RUN wget https://homepages.cae.wisc.edu/~ece533/images/lena.png && \
    wget https://homepages.cae.wisc.edu/~ece533/images/lena.bmp && \
    wget https://homepages.cae.wisc.edu/~ece533/images/lena.ppm && \
    wget https://www.gutenberg.org/files/74/74-0.txt

