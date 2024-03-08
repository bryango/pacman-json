//! This module defines the [`PackageInfo`] struct for serializing package
//! information, including functions to encode and decode relevant data.

use alpm::{decode_signature, Alpm, AlpmList, AlpmListMut, Dep, IntoAlpmListItem, Package};
use serde::{Serialize, Serializer};
use std::{borrow::Cow, fmt};

/// Formats an object to String with its Debug info
fn debug_format<T: fmt::Debug>(object: T) -> String {
    format!("{:?}", object)
}

// dump_pkg_full: https://gitlab.archlinux.org/pacman/pacman/-/blob/master/src/pacman/package.c

#[derive(Serialize)]
pub struct PackageInfo<'h> {
    // #[allow(dead_code)]
    // #[serde(skip)]
    // package: Package<'h>,
    repository: Option<&'h str>,
    name: &'h str,
    version: &'h str,
    description: Option<&'h str>,
    architecture: Option<&'h str>,
    url: Option<&'h str>,
    licenses: PacList<'h, &'h str>,
    groups: PacList<'h, &'h str>,
    provides: Vec<DepInfo<'h>>,
    depends_on: Vec<DepInfo<'h>>,
    optional_deps: Vec<DepInfo<'h>>,
    required_by: Cow<'h, [String]>,
    optional_for: Cow<'h, [String]>,
    conflicts_with: Vec<DepInfo<'h>>,
    replaces: Vec<DepInfo<'h>>,
    /// `download_size` and `compressed_size` are the same;
    /// both are `alpm_pkg_get_size` so we implement only one of them.
    download_size: i64,
    installed_size: i64,
    packager: Option<&'h str>,
    build_date: i64,
    install_date: Option<i64>,
    install_reason: String,
    install_script: bool,
    md5_sum: Option<&'h str>,
    sha_256_sum: Option<&'h str>,
    signatures: Option<&'h str>,
    /// `key_id` is set to None when initialized; it can be decoded on-demand
    /// with the [`Alpm`] handle with the [`decode_keyid`] function.
    key_id: Option<Vec<String>>,
    validated_by: String,
}

impl<'h> From<&Package<'h>> for PackageInfo<'h> {
    fn from(pkg: &Package<'h>) -> PackageInfo<'h> {
        eprintln!("pkg: {}", pkg.name());
        Self {
            // package: *pkg,
            repository: pkg.db().map(|db| db.name()),
            name: pkg.name(),
            version: pkg.version(),
            description: pkg.desc(),
            architecture: pkg.arch(),
            url: pkg.url(),
            licenses: PacList(pkg.licenses()),
            groups: PacList(pkg.groups()),
            provides: PacList(pkg.provides()).into(),
            depends_on: PacList(pkg.depends()).into(),
            optional_deps: PacList(pkg.optdepends()).into(),
            required_by: PacListMut(pkg.required_by()).into(),
            optional_for: PacListMut(pkg.optional_for()).into(),
            conflicts_with: PacList(pkg.conflicts()).into(),
            replaces: PacList(pkg.replaces()).into(),
            download_size: pkg.size(),
            installed_size: pkg.isize(),
            packager: pkg.packager(),
            build_date: pkg.build_date(),
            install_date: pkg.install_date(),
            install_reason: debug_format(pkg.reason()),
            install_script: pkg.has_scriptlet(),
            md5_sum: pkg.md5sum(),
            sha_256_sum: pkg.sha256sum(),
            signatures: pkg.base64_sig(),
            key_id: None,
            validated_by: debug_format(pkg.validation()),
        }
    }
}

#[derive(Serialize)]
struct DepInfo<'h> {
    name: &'h str,
    depmod: String,
    version: Option<&'h str>,
    description: Option<&'h str>,
    name_hash: u64,
}

impl<'h> From<Dep<'h>> for DepInfo<'h> {
    fn from(dep: Dep<'h>) -> DepInfo<'h> {
        Self {
            name: dep.name(),
            depmod: debug_format(dep.depmod()),
            version: dep.version().map(|x| x.as_str()),
            description: dep.desc(),
            name_hash: dep.name_hash(),
        }
    }
}

/// Decodes the signature & extracts the key ID
pub fn decode_keyid<'h>(handle: &'h Alpm, pkg_info: PackageInfo<'h>) -> PackageInfo<'h> {
    let sig = pkg_info.signatures.map(decode_signature);
    let res = if let Some(Ok(decoded)) = sig {
        Some(
            handle
                .extract_keyid(pkg_info.name, &decoded)
                .map_err(debug_format)
                .map(|x| x.into_iter().collect::<Vec<_>>())
                .map_or_else(|err| vec![err], |res| res),
        )
    } else {
        sig.map(debug_format).map(|err| vec![err])
    };
    PackageInfo {
        key_id: res,
        ..pkg_info
    }
}

/// Adds sync database info to the local package
#[allow(dead_code)]
pub fn add_sync_info<'h>(
    local_info: PackageInfo<'h>,
    sync_info: PackageInfo<'h>,
) -> PackageInfo<'h> {
    PackageInfo {
        repository: sync_info.repository,
        ..local_info
    }
}

/// Adds local database info to the sync package
pub fn add_local_info<'h>(
    local_info: PackageInfo<'h>,
    sync_info: PackageInfo<'h>,
) -> PackageInfo<'h> {
    PackageInfo {
        install_date: local_info.install_date,
        install_reason: local_info.install_reason,
        install_script: local_info.install_script,
        ..sync_info
    }
}

/// A type to enclose various lists, e.g. packages, licenses, ... that are
/// returned from alpm. This is a newtype around [`AlpmList`].
///
/// `impl Serialize for AlpmList` does not work due to rust "orphan rules";
/// see e.g. https://github.com/Ixrec/rust-orphan-rules.
struct PacList<'a, T>(AlpmList<'a, T>);
struct PacListMut<'a, T>(AlpmListMut<'a, T>)
where
    for<'b> T: IntoAlpmListItem<'a, 'b>;

impl<'a, T> Serialize for PacList<'a, T>
where
    T: IntoAlpmListItem<'a, 'a>,
    T::Borrow: Serialize,
{
    fn serialize<'h, S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let alpm_list = self.0;
        serializer.collect_seq(alpm_list.into_iter())
    }
}

/// Converts [`PacListMut<String>`] to a vec for easy serialization
impl From<PacListMut<'_, String>> for Cow<'_, [String]> {
    fn from(wrapper: PacListMut<'_, String>) -> Self {
        wrapper.0.into_iter().collect()
    }
}

/// Converts [`PacList<Dep>`] to a vec of [`DepInfo`] for easy serialization
impl<'a> From<PacList<'a, Dep<'a>>> for Vec<DepInfo<'a>> {
    fn from(wrapper: PacList<'a, Dep<'a>>) -> Self {
        wrapper.0.into_iter().map(|p| p.into()).collect()
    }
}
