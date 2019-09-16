% steganer(1) | steganer usage documentation

NAME
====

**steganer** — Library to hide a file inside another... or to recover it.

SYNOPSIS
========

| **steganer** FILE_HIDDEN HOST_FILE [**-x**|**--extract**] [**-h**|**--help**] [**-V**|**--version**]

DESCRIPTION
===========

If not run in extract mode then you are trying to hide FILE_HIDDEN inside HOST_FILE,
whereas if you set extract mode then you are trying to recover FILE_HIDDEN from
HOST_FILE.

Nowadays, steganer performs steganography over images (currently PNG, BMP and PPM 
images). Method used is to store chunks of data in Least Significant Bits of image
pixels. Only metadata steganer stores inside host images is hidden chunk size, so
you must know which extension hidden file has prior extraction. Hiding quality 
depends on image_size/hidden_data_size ratio, so host image should be much bigger 
than hidden data to keep hiding unnoticed. If you realize host image gets noise after
hiding then you should chose another bigger image as host.

Options
-------

-h, --help

:   Prints brief usage information.

-x, --extract

:   Run in extract mode.

-V, --version

:   Prints the current version number.

BUGS
====

Report issues at: <https://github.com/dante-signal31/steganer/issues>

AUTHOR
======

Dante Signal31 <dante.signal31@gmail.com>

SEE ALSO
========
Website: <https://github.com/dante-signal31/steganer>

COPYRIGHT
========

Copyright (c) 2019 Dante-Signal31 <dante.signal31@gmail.com>. All rights reserved.

Redistribution and use in source and binary forms, with or without modification, are permitted provided that the
following conditions are met:

    1. Redistributions of source code must retain the above copyright notice, this list of conditions and the
    following disclaimer.
    2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the
    following disclaimer in the documentation and/or other materials provided with the distribution.
    3. Neither the name of the copyright holder nor the names of its contributors may be used to endorse or
    promote products derived from this software without specific prior written permission.
    4. Redistributions of any form whatsoever must retain the following acknowledgment: 'This product includes
    software developed by the "Dante-Signal31" (dante.signal31@gmail.com).'

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES,
INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY,
WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
