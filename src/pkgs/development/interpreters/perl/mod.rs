use crate::{build::fetchurl::FetchUrl, stdenv::Stdenv};
use oxide_core::prelude::*;

pub struct Perl {
    pub stdenv: Stdenv,
    pub fetchurl: FetchUrl,
    pub zlib: LazyDrv,
    pub enable_threading: bool,
}

impl IntoDrv for Perl {
    fn into_drv(self) -> Drv {
        let version = "5.40.0";
        let mut configure_flags = vec![
            "-de",
            r#"-Dprefix=$out"#,
            r#"-Dman1dir=$out/share/man/man1"#,
            r#"-Dman3dir=$out/share/man/man3"#,
            "-Dcc=cc",
            "-Duseshrplib",
            "-Uinstallusrbinperl",
            "-Dinstallstyle=lib/perl5",
            "-Dlocincpth=$libcInc/include",
            "-Dloclibpth=$libcLib/lib",
        ];
        if self.enable_threading {
            configure_flags.push("-Dusethreads");
        }
        self.stdenv
            .make_derivation()
            .name("perl")
            .src(self.fetchurl.fetch(
                format!("mirror://cpan/src/5.0/perl-{version}.tar.gz"),
                hash!("sha512:fq6a0PymjshE-2APytfavrpCdpV8y0oXomFi8Icb-eRgqyENgsaf_CcLYkYQxxqnZO_pvr5pZzSQG016MKdTEg"),
            ))
            .input("STRICT_DEPS", "1")
            .out("out")
            .out("man")
            .out("devdoc")
            .patch(local_file!("patches/CVE-2024-56406.patch"))
            .patch(local_file!("patches/CVE-2025-40909.patch"))
            .patch(local_file!("patches/fix-build-with-only-C-locale-5.40.0.patch"))
            .patch(local_file!("patches/no-sys-dirs-5.40.0.patch"))
            .post_patch(
r#"postPatch =
substituteInPlace dist/PathTools/Cwd.pm \
    --replace "/bin/pwd" "$(type -P pwd)"
unset src
"#)
            // TODO: hacks to get around missing features
            .input("libcInc", self.stdenv.glibc.clone().unwrap().out("dev"))
            .input("libcLib", self.stdenv.glibc.clone().unwrap().out("lib"))
            .input("zlibDev", self.zlib.out("dev"))
            .input("zlibOut", self.zlib.out("out"))
            .input_if("cc", self.stdenv.cc.clone())
            .pre_phase(format!(r#"CONFIGURE_FLAGS={}\nCONFIGURE_SCRIPT="$shell ./Configure""#, configure_flags.join(" ")))
            // .configure_flags([
            //     "-de",
            //     r#"-Dprefix=${placeholder "out"}"#,
            //     r#"-Dman1dir=${placeholder "out"}/share/man/man1"#,
            //     r#"-Dman3dir=${placeholder "out"}/share/man/man3"#,
            //     "-Dcc=cc",
            //     "-Duseshrplib",
            //     "-Uinstallusrbinperl",
            //     "-Dinstallstyle=lib/perl5",
            //     "-Dlocincpth=${libcInc}/include",
            //     "-Dloclibpth=${libcLib}/lib",
            // ].join(" "))
            // .configure_script(r#"{self.stdenv.shell} ./Configure"#)
            .input("ENABLE_PARALLEL_BUILDING", "")
            .pre_configure(r#"
cat > config.over <<EOF
osvers="gnulinux"
myuname="nixpkgs"
myhostname="nixpkgs"
cf_by="nixpkgs"
cf_time="$(date -d "@$SOURCE_DATE_EPOCH")"
EOF

# Compress::Raw::Zlib should use our zlib package instead of the one
# included with the distribution
cat > ./cpan/Compress-Raw-Zlib/config.in <<EOF
BUILD_ZLIB   = False
INCLUDE      = $zlibDev/include
LIB          = $zlibOut/lib
OLD_ZLIB     = False
GZIP_OS_CODE = AUTO_DETECT
USE_ZLIB_NG  = False
ZLIB_INCLUDE = $zlibDev/include
ZLIB_LIB     = $zlibOut/lib
EOF"#)
            .input("SETUP_HOOK", local_file!("setup-hook.sh"))
            .post_install(
r#"# Remove dependency between "out" and "man" outputs.
rm "$out"/lib/perl5/*/*/.packlist

# Remove dependencies on glibc and gcc
sed "/ *libpth =>/c    libpth => ' '," \
  -i "$out"/lib/perl5/*/*/Config.pm
# TODO: removing those paths would be cleaner than overwriting with nonsense.
substituteInPlace "$out"/lib/perl5/*/*/Config_heavy.pl \
  --replace "$libcInc" /no-such-path \
  --replace "$cc" else "/no-such-path" /no-such-path \
  --replace "$man" /no-such-path"#)
            .build()
    }
}
