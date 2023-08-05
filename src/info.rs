
use std::fmt;
use alpm::{Package, Dep, AlpmList, AlpmListMut, Alpm, decode_signature};
use serde::{Serialize, Serializer, ser::SerializeSeq};


fn debug_format<T: fmt::Debug>(object: T) -> String {
    format!("{:?}", object)
}

// dump_pkg_full: https://gitlab.archlinux.org/pacman/pacman/-/blob/master/src/pacman/package.c

#[derive(Serialize)]
pub struct PackageInfo<'h> {
    // #[serde(skip)]
    // package: Package<'h>,
    repository: Option<&'h str>,
    name: &'h str,
    version: &'h str,
    description: Option<&'h str>,
    architecture: Option<&'h str>,
    url: Option<&'h str>,
    #[serde(serialize_with = "serialize_alpm_list_str")]
    licenses: AlpmList<'h, &'h str>,
    #[serde(serialize_with = "serialize_alpm_list_str")]
    groups: AlpmList<'h, &'h str>,
    #[serde(serialize_with = "serialize_alpm_list_dep")]
    provides: AlpmList<'h, Dep<'h>>,
    #[serde(serialize_with = "serialize_alpm_list_dep")]
    depends_on: AlpmList<'h, Dep<'h>>,
    #[serde(serialize_with = "serialize_alpm_list_dep")]
    optional_deps: AlpmList<'h, Dep<'h>>,
    #[serde(serialize_with = "serialize_alpm_list_mut_string")]
    required_by: AlpmListMut<'h, String>,
    #[serde(serialize_with = "serialize_alpm_list_mut_string")]
    optional_for: AlpmListMut<'h, String>,
    #[serde(serialize_with = "serialize_alpm_list_dep")]
    conflicts_with: AlpmList<'h, Dep<'h>>,
    #[serde(serialize_with = "serialize_alpm_list_dep")]
    replaces: AlpmList<'h, Dep<'h>>,
    download_size: i64,
    // ^ `compressed_size` is the same as `download_size`
    // both are `alpm_pkg_get_size`
    installed_size: i64,
    packager: Option<&'h str>,
    build_date: i64,
    install_date: Option<i64>,
    install_reason: String,
    install_script: bool,
    md5_sum: Option<&'h str>,
    sha_256_sum: Option<&'h str>,
    signatures: Option<&'h str>,
    key_id: Option<Vec<String>>,
    validated_by: String,
}

impl<'h> From<&Package<'h>> for PackageInfo<'h> {
    fn from(pkg: &Package<'h>) -> PackageInfo<'h> {
        Self {
            // package: *pkg,
            repository: pkg.db().map(|db| db.name()),
            name: pkg.name(),
            version: pkg.version(),
            description: pkg.desc(),
            architecture: pkg.arch(),
            url: pkg.url(),
            licenses: pkg.licenses(),
            groups: pkg.groups(),
            provides: pkg.provides(),
            depends_on: pkg.depends(),
            optional_deps: pkg.optdepends(),
            required_by: pkg.required_by(),
            optional_for: pkg.optional_for(),
            conflicts_with: pkg.conflicts(),
            replaces: pkg.replaces(),
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
            validated_by: format!("{:?}", pkg.validation())
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

impl<'h> From<&Dep<'h>> for DepInfo<'h> {
    fn from(dep: &Dep<'h>) -> DepInfo<'h> {
        Self {
            name: dep.name(),
            depmod: format!("{:?}", dep.depmod()),
            version: dep.version().map(|x| x.as_str()),
            description: dep.desc(),
            name_hash: dep.name_hash()
        }
    }
}

/// Decodes the signature & extracts the key ID
pub fn decode_keyid<'h>(handle: &'h Alpm, pkg_info: PackageInfo<'h>) -> PackageInfo<'h> {
    let sig = pkg_info.signatures.map(decode_signature);
    let res =
        if let Some(Ok(decoded)) = sig {
            Some(
                handle.extract_keyid(pkg_info.name, &decoded)
                .map_err(debug_format)
                .map(|x| x.into_iter().collect::<Vec<_>>())
                .map_or_else(|err| vec![err], |res| res)
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
pub fn add_sync_info<'h>(local_info: PackageInfo<'h>, sync_info: PackageInfo<'h>) -> PackageInfo<'h> {
    PackageInfo {
        repository: sync_info.repository,
        ..local_info
    }
}

/// Adds local database info to the sync package
pub fn add_local_info<'h>(local_info: PackageInfo<'h>, sync_info: PackageInfo<'h>) -> PackageInfo<'h> {
    PackageInfo {
        install_date: local_info.install_date,
        install_reason: local_info.install_reason,
        install_script: local_info.install_script,
        ..sync_info
    }
}


// implement the `serialize` functions used above

fn serialize_alpm_list_str<S>(alpm_list: &AlpmList<'_, &str>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.collect_seq(alpm_list.iter())
}

fn serialize_alpm_list_mut_string<S>(alpm_list: &AlpmListMut<'_, String>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.collect_seq(alpm_list.iter())
}

fn serialize_alpm_list_dep<S>(alpm_list: &AlpmList<'_, Dep<'_>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(alpm_list.len()))?;
    for item in alpm_list {
        seq.serialize_element(&DepInfo::from(&item))?;
    }
    seq.end()
}
