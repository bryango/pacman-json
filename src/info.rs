
use std::fmt;
use alpm::{Package, AlpmList, Dep, AlpmListMut};
use serde::{Serialize, Serializer, ser::SerializeSeq};


fn debug_format<T: fmt::Debug>(object: T) -> String {
    format!("{:?}", object)
}

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
    signatures: Result<String, String>,
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
            signatures: pkg.sig().map(debug_format).map_err(debug_format),
            validated_by: format!("{:?}", pkg.validation())
        }
    }
}

#[derive(Serialize)]
pub struct DepInfo<'h> {
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
