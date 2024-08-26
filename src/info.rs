//! A module that defines the [`PackageInfo`] struct for serializing package
//! information, including functions to encode and decode relevant data.

use alpm::{decode_signature, Alpm, AlpmList, Dep, IntoAlpmListItem, Package};
use serde::Serialize;
use std::fmt::Debug;

use crate::reverse_deps::{ReverseDeps, ReverseDepsDatabase, ReverseDepsMap};

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
    /// [`add_reverse_deps`]: PackageInfo::add_reverse_deps
    ///
    pub required_by: &'h ReverseDeps,
    pub optional_for: &'h ReverseDeps,
    pub required_by_make: &'h ReverseDeps,
    pub required_by_check: &'h ReverseDeps,

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
    /// with the [`Alpm`] handle using the
    /// [`decode_keyid`][PackageInfo::decode_keyid] method.
    ///
    pub key_id: Option<Vec<Box<str>>>,
    pub validated_by: Box<str>,
    pub sync_with: Option<Box<Self>>,
}

impl<'h> From<&'h Package> for PackageInfo<'h> {
    /// Converts an [`alpm::Package`] to a [`PackageInfo`] containing the
    /// relevant information, to be serialized.
    fn from(pkg: &'h Package) -> Self {
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
            required_by: ReverseDeps::NONE,
            optional_for: ReverseDeps::NONE,
            required_by_make: ReverseDeps::NONE,
            required_by_check: ReverseDeps::NONE,
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

impl<'a> PackageInfo<'a> {
    /// Generates [`PackageInfo`] from an [`Alpm::syncdbs`] package,
    /// decoding the associated signature key ID in the process.
    pub fn from_sync_pkg(handle: &'a Alpm, pkg: &'a Package) -> Self {
        Self::from(pkg).decode_keyid(handle)
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
    fn from(dep: &'h Dep) -> Self {
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
    /// Tries to decode the signature of an [`Alpm::syncdbs`] package with an [`Alpm`]
    /// handle and return the key ID.
    fn get_keyid(&self, handle: &'h Alpm) -> anyhow::Result<Vec<Box<str>>> {
        let decoded = match self.signatures {
            None => anyhow::bail!("signatures not found for {self:?}"),
            Some(sig) => decode_signature(sig)?,
        };
        let res = handle
            .extract_keyid(self.name, &decoded)?
            .into_iter()
            .map(|x| x.into_boxed_str())
            .collect::<Vec<_>>();
        Ok(res)
    }

    /// Decodes the signature of an [`Alpm::syncdbs`] package with an [`Alpm`]
    /// handle and extracts the key ID, writing possible errors into the final
    /// [`Vec<Box<str>>`].
    pub fn decode_keyid(self, handle: &'h Alpm) -> Self {
        let key_id = self.get_keyid(handle).unwrap_or_else(|x| vec![x.format()]);
        Self {
            key_id: Some(key_id),
            ..self
        }
    }

    /// Adds sync database info to a local package
    pub fn add_sync_info(self, sync_info: Self) -> Self {
        let local_info = self;
        PackageInfo {
            sync_with: Some(Box::new(sync_info)),
            ..local_info
        }
    }

    /// Adds local database info to a sync package
    pub fn add_local_info(self, local_info: Self) -> Self {
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

    /// Adds reverse dependencies information to a package
    pub fn add_reverse_deps(self, reverse_deps: &'h ReverseDepsDatabase) -> Self {
        let get =
            |rev_deps_map: &'h ReverseDepsMap| rev_deps_map.get(self.name).unwrap_or_default();
        PackageInfo {
            required_by: get(&reverse_deps.required_by),
            optional_for: get(&reverse_deps.optional_for),
            required_by_make: get(&reverse_deps.required_by_make),
            required_by_check: get(&reverse_deps.required_by_check),
            ..self
        }
    }
}

/// A newtype [`Vec`] to enclose various lists, e.g. packages, licenses, ...
/// returned from alpm. This serves as a proxy for [`AlpmList`] to be
/// [`Serialize`]d; conversions from [`AlpmList`] are implemented.
///
/// Note that the naive <code>impl [Serialize] for [AlpmList]</code>
/// does not work due to rust "orphan rules";
/// see e.g. <https://github.com/Ixrec/rust-orphan-rules>.
///
#[derive(Serialize, Clone, Debug, derive_more::IntoIterator, derive_more::From)]
pub struct PacList<T>(Vec<T>);

impl<'a, T: IntoAlpmListItem> From<AlpmList<'a, T>> for PacList<T> {
    fn from(alpm_list: AlpmList<'a, T>) -> Self {
        let vector: Vec<_> = alpm_list.into_iter().collect();
        Self(vector)
    }
}

impl<'a> From<AlpmList<'a, &'a Dep>> for PacList<DepInfo<'a>> {
    fn from(alpm_list: AlpmList<'a, &'a Dep>) -> Self {
        let vector: Vec<_> = alpm_list
            .into_iter()
            .map(|dep| DepInfo::from(dep))
            .collect();
        Self(vector)
    }
}
