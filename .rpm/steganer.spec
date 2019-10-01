%define __spec_install_post %{nil}
%define __os_install_post %{_dbpath}/brp-compress
%define debug_package %{nil}

Name: steganer
Summary: Library to hide a file inside another... or to recover it.
Version: @@VERSION@@
Release: @@RELEASE@@
License: Copyright (c) 2019 Dante-Signal31 &lt;dante.signal31@gmail.com&gt;. All rights reserved.
Group: Applications/System
Source0: %{name}-%{version}.tar.gz
URL: https://github.com/dante-signal31/steganer

BuildRoot: %{_tmppath}/%{name}-%{version}-%{release}-root

%description
%{summary}

%prep
%setup -q

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
mkdir -p %{buildroot}/usr/share/man/man1/
mkdir -p %{buildroot}/usr/share/doc/steganer/
cp %{buildroot}/../../../../../man/steganer.1.gz %{buildroot}/usr/share/man/man1/steganer.1.gz
cp %{buildroot}/../../../../../README.md %{buildroot}/usr/share/doc/steganer/README.md
cp -a * %{buildroot}

%clean
rm -rf %{buildroot}

%files
/usr/bin/steganer
/usr/share/man/man1/steganer.1.gz
/usr/share/doc/steganer/README.md

%defattr(-,root,root,-)
%{_bindir}/*
