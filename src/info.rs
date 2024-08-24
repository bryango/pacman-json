//! A module that defines the [`PackageInfo`] struct for serializing package
//! information, including functions to encode and decode relevant data.

use alpm::{decode_signature, Alpm, AlpmList, Dep, IntoAlpmListItem, Package};
use serde::Serialize;
use std::{collections::BTreeSet, fmt::Debug};

use crate::reverse_deps::{RevDepsMap, ReverseDependencyMaps};

static EMPTY_REV_DEPS: BTreeSet<String> = BTreeSet::new();

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

/// A wrapper of the relevant information of a pacman [`Package`]
/// for ease of serialization by [`serde`].
/// The reference implementation in th official `pacman` package
/// is given by the `dump_pkg_full` function in [`package.c`].
///
/// [`package.c`]: https://gitlab.archlinux.org/pacman/pacman/-/blob/master/src/pacman/package.c
///
#[derive(Serialize, Clone, Debug)]
#[non_exhaustive]
pub struct PackageInfo<'h> {
    // #[allow(dead_code)]
    // #[serde(skip)]
    // package: Package<'h>,
    pub repository: Option<&'h str>,
    pub name: &'h str,
    pub version: &'h str,
    pub description: Option<&'h str>,
    pub architecture: Option<&'h str>,
    pub url: Option<&'h str>,
    pub licenses: PacList<&'h str>,
    pub groups: PacList<&'h str>,
    pub provides: PacList<DepInfo<'h>>,
    pub depends_on: PacList<DepInfo<'h>>,
    pub optional_deps: PacList<DepInfo<'h>>,
    pub makedepends: PacList<DepInfo<'h>>,
    pub checkdepends: PacList<DepInfo<'h>>,
    pub conflicts_with: PacList<DepInfo<'h>>,
    pub replaces: PacList<DepInfo<'h>>,

    /// Note that [`required_by`] and similarly, [`optional_for`]
    /// and <code>required_by_{[make],[check]}</code>
    /// are reverse dependencies; they are
    /// computed on demand with the [`add_reverse_deps`] function.
    ///
    /// [`required_by`]: PackageInfo::required_by
    /// [`optional_for`]: PackageInfo::optional_for
    /// [make]: PackageInfo::required_by_make
    /// [check]: PackageInfo::required_by_check
    ///
    pub required_by: &'h BTreeSet<String>,
    pub optional_for: &'h BTreeSet<String>,
    pub required_by_make: &'h BTreeSet<String>,
    pub required_by_check: &'h BTreeSet<String>,

    /// Note that [`download_size`] and `compressed_size` are
    /// essentially the same; both are `alpm_pkg_get_size` so we implement
    /// only one of them.
    ///
    /// [`download_size`]: PackageInfo::download_size
    ///
    pub download_size: i64,
    pub installed_size: i64,
    pub packager: Option<&'h str>,
    pub build_date: i64,
    pub install_date: Option<i64>,
    pub install_reason: Box<str>,
    pub install_script: bool,
    pub md5_sum: Option<&'h str>,
    pub sha_256_sum: Option<&'h str>,
    pub signatures: Option<&'h str>,

    /// Note that [`key_id`][PackageInfo::key_id] is set to None
    /// when initialized; it can be decoded on-demand
    /// with the [`Alpm`] handle with the [`decode_keyid`] function.
    pub key_id: Option<Vec<Box<str>>>,
    pub validated_by: Box<str>,
    pub sync_with: Option<Box<PackageInfo<'h>>>,
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
            required_by: &EMPTY_REV_DEPS,
            optional_for: &EMPTY_REV_DEPS,
            required_by_make: &EMPTY_REV_DEPS,
            required_by_check: &EMPTY_REV_DEPS,
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

/// A wrapper of the information of a pacman dependency [`Dep`]
/// for ease of serialization by [`serde`].
#[derive(Serialize, Clone, Debug)]
pub struct DepInfo<'h> {
    pub dep_string: String,
    pub name: &'h str,
    pub depmod: Box<str>,
    pub version: Option<&'h str>,
    pub description: Option<&'h str>,
    pub name_hash: u64,
    pub satisfier: Option<String>,
}

impl<'h> From<&'h Dep> for DepInfo<'h> {
    fn from(dep: &'h Dep) -> DepInfo<'h> {
        Self {
            dep_string: dep.to_string(),
            name: dep.name(),
            depmod: dep.depmod().format(),
            version: dep.version().map(|x| x.as_str()),
            description: dep.desc(),
            name_hash: dep.name_hash(),
            satisfier: None,
        }
    }
}

impl<'h> PackageInfo<'h> {
    /// Decodes the signature & extracts the key ID
    pub fn decode_keyid(self, handle: &'h Alpm) -> PackageInfo<'h> {
        let sig = match self.signatures {
            None => return self,
            Some(sig) => decode_signature(sig),
        };
        let res = match sig {
            Err(err) => vec![err.format()],
            Ok(decoded) => handle.extract_keyid(self.name, &decoded).map_or_else(
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
            ..self
        }
    }

    /// Adds sync database info to a local package
    pub fn add_sync_info(self, sync_info: PackageInfo<'h>) -> PackageInfo<'h> {
        let local_info = self;
        PackageInfo {
            sync_with: Some(Box::new(sync_info)),
            ..local_info
        }
    }

    /// Adds local database info to a sync package
    pub fn add_local_info(self, local_info: PackageInfo<'h>) -> PackageInfo<'h> {
        let sync_info = self;
        let reason = local_info.install_reason.clone(); // otherwise partial move
        PackageInfo {
            install_date: local_info.install_date,
            install_reason: reason,
            install_script: local_info.install_script,
            sync_with: Some(Box::new(local_info)),
            ..sync_info
        }
    }
}

/// Adds reverse dependencies information to a package
pub fn add_reverse_deps<'h>(
    pkg_info: PackageInfo<'h>,
    reverse_deps: &'h ReverseDependencyMaps,
) -> PackageInfo<'h> {
    let get =
        |rev_deps_map: &'h RevDepsMap| rev_deps_map.get(pkg_info.name).unwrap_or(&EMPTY_REV_DEPS);
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
/// see e.g. <https://github.com/Ixrec/rust-orphan-rules>.
#[derive(Serialize, Clone, Debug, derive_more::Deref, derive_more::From)]
pub struct PacList<T>(Vec<T>);

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
