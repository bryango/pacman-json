//! This module defines the [`PackageInfo`] struct for serializing package
//! information, including functions to encode and decode relevant data.

use alpm::{decode_signature, Alpm, AlpmList, Dep, IntoAlpmListItem, Package};
use serde::Serialize;
use std::{collections::HashSet, fmt::Debug};

use crate::reverse_deps::{RevDepsMap, ReverseDependencyMaps};

trait DebugFormat {
    /// Formats an object to [`Box<str>`] with its [`Debug`] info
    fn format(&self) -> Box<str>
    where
        Self: Debug,
    {
        format!("{:?}", self).into()
    }
}

/// Blanket implementation of [`DebugFormat`] for all types with [`Debug`].
/// This is efficient, as it seems that the implementations for Enums will
/// be inlined in release mode. Nice!
impl<T: Debug> DebugFormat for T {}

// dump_pkg_full: https://gitlab.archlinux.org/pacman/pacman/-/blob/master/src/pacman/package.c

/// This is a wrapper of the relevant information of a pacman [`Package`]
/// for ease of serialization by [`serde`].
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
    licenses: PacList<&'h str>,
    groups: PacList<&'h str>,
    provides: PacList<DepInfo<'h>>,
    depends_on: PacList<DepInfo<'h>>,
    optional_deps: PacList<DepInfo<'h>>,
    makedepends: PacList<DepInfo<'h>>,
    checkdepends: PacList<DepInfo<'h>>,
    conflicts_with: PacList<DepInfo<'h>>,
    replaces: PacList<DepInfo<'h>>,

    /// [`PackageInfo::required_by`] and similarly, `optional_for`
    /// and `required_by_{make,check}` are reverse dependencies; they are
    /// computed on demand with the [`add_reverse_deps`] function.
    required_by: HashSet<String>,
    optional_for: HashSet<String>,
    required_by_make: HashSet<String>,
    required_by_check: HashSet<String>,

    /// `download_size` and `compressed_size` are the same;
    /// both are `alpm_pkg_get_size` so we implement only one of them.
    download_size: i64,
    installed_size: i64,
    packager: Option<&'h str>,
    build_date: i64,
    install_date: Option<i64>,
    install_reason: Box<str>,
    install_script: bool,
    md5_sum: Option<&'h str>,
    sha_256_sum: Option<&'h str>,
    signatures: Option<&'h str>,

    /// `key_id` is set to None when initialized; it can be decoded on-demand
    /// with the [`Alpm`] handle with the [`decode_keyid`] function.
    key_id: Option<Vec<Box<str>>>,
    validated_by: Box<str>,
    sync_with: Option<Box<PackageInfo<'h>>>,
}

impl<'h> From<&'h Package> for PackageInfo<'h> {
    /// Converts an alpm [`Package`] to a [`PackageInfo`] containing the
    /// relevant information, to be serialized.
    fn from(pkg: &'h Package) -> PackageInfo<'h> {
        let db = pkg.db().map(|db| db.name());
        let name = pkg.name();
        // eprintln!("{:?}: {}", db, name);

        Self {
            // package: *pkg,
            repository: db,
            name: name,
            version: pkg.version(),
            description: pkg.desc(),
            architecture: pkg.arch(),
            url: pkg.url(),
            licenses: pkg.licenses().into(),
            groups: pkg.groups().into(),
            provides: pkg.provides().into(),
            depends_on: pkg.depends().into(),
            optional_deps: pkg.optdepends().into(),
            makedepends: pkg.makedepends().into(),
            checkdepends: pkg.checkdepends().into(),
            required_by: [].into(),
            optional_for: [].into(),
            required_by_make: [].into(),
            required_by_check: [].into(),
            conflicts_with: pkg.conflicts().into(),
            replaces: pkg.replaces().into(),
            download_size: pkg.size(),
            installed_size: pkg.isize(),
            packager: pkg.packager(),
            build_date: pkg.build_date(),
            install_date: pkg.install_date(),
            install_reason: pkg.reason().format(),
            install_script: pkg.has_scriptlet(),
            md5_sum: pkg.md5sum(),
            sha_256_sum: pkg.sha256sum(),
            signatures: pkg.base64_sig(),
            key_id: None,
            validated_by: pkg.validation().format(),
            sync_with: None,
        }
    }
}

#[derive(Serialize)]
struct DepInfo<'h> {
    name: &'h str,
    depmod: Box<str>,
    version: Option<&'h str>,
    description: Option<&'h str>,
    name_hash: u64,
    package_info: Option<PackageInfo<'h>>,
}

impl<'h> From<&'h Dep> for DepInfo<'h> {
    fn from(dep: &'h Dep) -> DepInfo<'h> {
        Self {
            name: dep.name(),
            depmod: dep.depmod().format(),
            version: dep.version().map(|x| x.as_str()),
            description: dep.desc(),
            name_hash: dep.name_hash(),
            package_info: None,
        }
    }
}

/// Decodes the signature & extracts the key ID
pub fn decode_keyid<'h>(handle: &'h Alpm, pkg_info: PackageInfo<'h>) -> PackageInfo<'h> {
    let sig = match pkg_info.signatures {
        None => return pkg_info,
        Some(sig) => decode_signature(sig),
    };
    let res = match sig {
        Err(err) => vec![err.format()],
        Ok(decoded) => handle.extract_keyid(pkg_info.name, &decoded).map_or_else(
            |err| vec![err.format()],
            |keys| {
                keys.into_iter()
                    .map(|x| x.into_boxed_str())
                    .collect::<Vec<_>>()
            },
        ),
    };
    PackageInfo {
        key_id: Some(res),
        ..pkg_info
    }
}

/// Adds sync database info to the local package
pub fn add_sync_info<'h>(
    local_info: PackageInfo<'h>,
    sync_info: PackageInfo<'h>,
) -> PackageInfo<'h> {
    PackageInfo {
        sync_with: Some(Box::new(sync_info)),
        ..local_info
    }
}

/// Adds local database info to the sync package
pub fn add_local_info<'h>(
    local_info: PackageInfo<'h>,
    sync_info: PackageInfo<'h>,
) -> PackageInfo<'h> {
    let reason = local_info.install_reason.clone(); // otherwise partial move
    PackageInfo {
        install_date: local_info.install_date,
        install_reason: reason,
        install_script: local_info.install_script,
        sync_with: Some(Box::new(local_info)),
        ..sync_info
    }
}

/// Adds reverse dependencies info
pub fn add_reverse_deps<'h>(
    pkg_info: PackageInfo<'h>,
    reverse_deps: &'h ReverseDependencyMaps,
) -> PackageInfo<'h> {
    let get = |rev_deps_map: &RevDepsMap| {
        rev_deps_map
            .get(pkg_info.name)
            .map_or(HashSet::new(), |x| x.to_owned())
    };
    PackageInfo {
        required_by: get(&reverse_deps.required_by),
        optional_for: get(&reverse_deps.optional_for),
        required_by_make: get(&reverse_deps.required_by_make),
        required_by_check: get(&reverse_deps.required_by_check),
        ..pkg_info
    }
}

/// A newtype [`Vec`] to enclose various lists, e.g. packages, licenses, ...
/// returned from alpm. Conversions from [`AlpmList`] are implemented.
///
/// `impl Serialize for AlpmList` does not work due to rust "orphan rules";
/// see e.g. https://github.com/Ixrec/rust-orphan-rules.
#[derive(Serialize)]
struct PacList<T>(Vec<T>);

impl<'a, T: IntoAlpmListItem> From<AlpmList<'a, T>> for PacList<T> {
    fn from(alpm_list: AlpmList<'a, T>) -> Self {
        let vector: Vec<_> = alpm_list.into_iter().collect();
        PacList(vector)
    }
}

impl<'a> From<AlpmList<'a, &'a Dep>> for PacList<DepInfo<'a>> {
    fn from(alpm_list: AlpmList<'a, &'a Dep>) -> Self {
        let vector: Vec<_> = alpm_list
            .into_iter()
            .map(|dep| DepInfo::from(dep))
            .collect();
        PacList(vector)
    }
}
