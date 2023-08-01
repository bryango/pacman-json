
use alpm::{Package, Db, AlpmList, Dep, AlpmListMut, PackageReason, Signature, PackageValidation, Error};
use serde::{Serialize, Serializer, ser::SerializeSeq};

#[derive(Serialize)]
pub struct PackageInfo<'h> {
    #[serde(skip)]
    package: Package<'h>,
    #[serde(serialize_with = "serialize_db")]
    repository: Option<Db<'h>>,
    name: &'h str,
    version: &'h str,
    description: Option<&'h str>,
    architecture: Option<&'h str>,
    url: Option<&'h str>,
    #[serde(serialize_with = "serialize_alpm_list_str")]
    licenses: AlpmList<'h, &'h str>,
    #[serde(serialize_with = "serialize_alpm_list_str")]
    groups: AlpmList<'h, &'h str>,
    provides: AlpmList<'h, Dep<'h>>,
    depends_on: AlpmList<'h, Dep<'h>>,
    optional_deps: AlpmList<'h, Dep<'h>>,
    #[serde(serialize_with = "serialize_alpm_list_mut_string")]
    required_by: AlpmListMut<'h, String>,
    #[serde(serialize_with = "serialize_alpm_list_mut_string")]
    optional_for: AlpmListMut<'h, String>,
    conflicts_with: AlpmList<'h, Dep<'h>>,
    replaces: AlpmList<'h, Dep<'h>>,
    download_size: i64,
    // ^ `compressed_size` is the same as `download_size`
    // both are `alpm_pkg_get_size`
    installed_size: i64,
    packager: Option<&'h str>,
    build_date: i64,
    install_date: Option<i64>,
    install_reason: PackageReason,
    install_script: bool,
    md5_sum: Option<&'h str>,
    sha_256_sum: Option<&'h str>,
    signatures: Result<Signature, Error>,
    validated_by: PackageValidation,
}

trait ToInfo {
    fn to_info(&self) -> PackageInfo;
}

impl ToInfo for Package<'_> {
    fn to_info(&self) -> PackageInfo {
        PackageInfo {
            package: *self,
            repository: self.db(),
            name: self.name(),
            version: self.version(),
            description: self.desc(),
            architecture: self.arch(),
            url: self.url(),
            licenses: self.licenses(),
            groups: self.groups(),
            provides: self.provides(),
            depends_on: self.depends(),
            optional_deps: self.optdepends(),
            required_by: self.required_by(),
            optional_for: self.optional_for(),
            conflicts_with: self.conflicts(),
            replaces: self.replaces(),
            download_size: self.size(),
            installed_size: self.isize(),
            packager: self.packager(),
            build_date: self.build_date(),
            install_date: self.install_date(),
            install_reason: self.reason(),
            install_script: self.has_scriptlet(),
            md5_sum: self.md5sum(),
            sha_256_sum: self.sha256sum(),
            signatures: self.sig(),
            validated_by: self.validation()
        }
    }
}

fn serialize_db<S>(opt_db: &Option<Db>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match *opt_db {
        Some(db) => serializer.serialize_str(db.name()),
        None => serializer.serialize_none(),
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

// fn serialize_alpm_list_dep<S>(alpm_list: &AlpmList<'_, Dep<'_>>, serializer: S) -> Result<S::Ok, S::Error>
// where
//     S: Serializer,
// {
//     let mut seq = serializer.serialize_seq(Some(alpm_list.len()))?;
//     for item in alpm_list {
//         seq.serialize_element(item)?;
//     }
//     seq.end()
// }
