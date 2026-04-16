use crate::{
    auth::AuthConfig,
    shared::{AppError, AppResult},
    users::UserRole,
};

use ldap3::{LdapConnAsync, LdapConnSettings, Scope, SearchEntry};
use std::time::Duration;
use sword::prelude::*;

#[injectable]
pub struct LdapClient {
    config: AuthConfig,
}

pub struct LdapUserInfo {
    pub email: String,
    pub name: String,
    pub role: UserRole,
}

impl LdapClient {
    /// Autentica un usuario contra LDAP
    ///
    /// Flujo:
    /// 1. Conectar a LDAP (anónimo)
    /// 2. Bind como ADMIN para buscar el usuario
    /// 3. Obtener el DN completo del usuario (incluyendo OU)
    /// 4. Desconectar como admin
    /// 5. Reconectar con credenciales del usuario
    /// 6. Bind como el usuario (valida su contraseña)
    /// 7. Obtener información del usuario
    /// 8. Desconectar
    pub async fn authenticate(&self, username: &str, password: &str) -> AppResult<LdapUserInfo> {
        // Paso 1: Conectar a LDAP
        let settings = LdapConnSettings::new()
            .set_conn_timeout(Duration::from_secs(5))
            .set_no_tls_verify(true);

        let (conn, mut ldap) =
            LdapConnAsync::with_settings(settings.clone(), &self.config.ldap_url)
                .await
                .inspect_err(|e| {
                    tracing::error!("[!] Error de conexión LDAP: {e}");
                })?;

        ldap3::drive!(conn);

        // Paso 2: Autenticarse como ADMIN
        let admin_dn = format!(
            "{},{}",
            self.config.ldap_admin_user, self.config.ldap_base_dn
        );

        tracing::debug!("[*] Autenticando como admin: {}", admin_dn);

        ldap.simple_bind(&admin_dn, &self.config.ldap_admin_password)
            .await
            .inspect_err(|e| {
                tracing::error!("[!] Error de conexión durante bind admin: {e}");
            })?
            .success()
            .inspect_err(|e| {
                tracing::error!("[!] Error de autenticación como admin LDAP: {e}");
            })?;

        tracing::debug!("[✓] Autenticación admin exitosa");

        // Paso 3: Buscar el usuario completo (con OU incluida)
        let user_dn = self
            .find_user_dn(&mut ldap, username)
            .await
            .inspect_err(|e| {
                tracing::error!("[!] No se pudo encontrar usuario {}: {}", username, e);
            })?;

        tracing::debug!("[*] Usuario encontrado con DN: {}", user_dn);

        // Paso 4: Desconectar como admin
        ldap.unbind().await.inspect_err(|e| {
            tracing::warn!("[!] Error al desautenticar admin: {e}");
        })?;

        // Paso 5: Reconectar para autenticarse como el usuario
        let (conn, mut ldap) = LdapConnAsync::with_settings(settings, &self.config.ldap_url)
            .await
            .inspect_err(|e| {
                tracing::error!("[!] Error de reconexión LDAP: {e}");
            })?;

        ldap3::drive!(conn);

        // Paso 6: Bind como el usuario (valida su contraseña)
        tracing::debug!("[*] Validando contraseña del usuario: {}", username);

        ldap.simple_bind(&user_dn, password)
            .await
            .inspect_err(|e| {
                tracing::error!("[!] Error de conexión durante bind de usuario: {e}");
            })?
            .success()
            .inspect_err(|e| {
                tracing::warn!("[!] Autenticación fallida para usuario {}: {e}", username);
            })?;

        tracing::info!("[✓] Usuario {} autenticado exitosamente", username);

        // Paso 7: Obtener información del usuario
        let user_info = self.find_user_info(&mut ldap, &user_dn, username).await?;

        // Paso 8: Desconectar
        ldap.unbind().await.inspect_err(|e| {
            tracing::warn!("[!] Error al desautenticar usuario: {e}");
        })?;

        Ok(user_info)
    }

    /// Busca el DN completo de un usuario en LDAP
    ///
    /// Requiere estar autenticado como admin
    /// Busca en toda la estructura LDAP incluyendo OUs
    async fn find_user_dn(&self, conn: &mut ldap3::Ldap, username: &str) -> AppResult<String> {
        let filter = format!("(uid={username})");

        tracing::debug!("[*] Buscando usuario con filtro: {}", filter);

        let (results, _) = conn
            .search(
                &self.config.ldap_base_dn,
                Scope::Subtree,
                &filter,
                vec!["dn"],
            )
            .await?
            .success()?;

        if results.is_empty() {
            tracing::error!("[!] Usuario no encontrado en LDAP: {}", username);
            return Err(AppError::LdapUsernameNotFound(username.to_string()));
        }

        let dn = results
            .iter()
            .map(|entry| SearchEntry::construct(entry.clone()).dn)
            .collect::<Vec<String>>();

        if dn.len() > 1 {
            tracing::warn!(
                "[LDAP] Se encontraron múltiples DNs para el usuario {}: {:#?}",
                username,
                dn
            );
        }

        let dn = dn[0].clone();

        tracing::debug!("[✓] DN encontrado: {}", dn);

        Ok(dn)
    }

    /// Obtiene la información del usuario (email, nombre, rol)
    ///
    /// Requiere estar autenticado como el usuario
    async fn find_user_info(
        &self,
        conn: &mut ldap3::Ldap,
        user_dn: &str,
        username: &str,
    ) -> AppResult<LdapUserInfo> {
        let filter = "(|(objectClass=inetOrgPerson)(objectClass=posixAccount))";

        tracing::debug!("[*] Buscando atributos del usuario: {}", user_dn);

        let (results, _) = conn
            .search(
                user_dn,
                Scope::Base,
                filter,
                vec!["mail", "cn", "gidNumber"],
            )
            .await?
            .success()?;

        let user_info = results
            .into_iter()
            .next()
            .and_then(|entry| {
                let entry = SearchEntry::construct(entry);

                // Email es requerido
                let email = entry.attrs.get("mail").and_then(|m| m.first().cloned())?;

                // Nombre (fallback a username si no existe)
                let name = entry
                    .attrs
                    .get("cn")
                    .and_then(|n| n.first().cloned())
                    .unwrap_or_else(|| username.to_string());

                // Rol basado en gidNumber
                let role = match entry.attrs.get("gidNumber").and_then(|g| g.first()) {
                    Some(gid) if gid == "600" => {
                        tracing::debug!("[*] Usuario {} es Func (gid=600)", username);
                        UserRole::Func
                    }

                    Some(gid) if gid == "500" => {
                        // Alternativo si tienes gid=500 para students
                        tracing::debug!("[*] Usuario {} es Student (gid=500)", username);
                        UserRole::Student
                    }
                    Some(gid) => {
                        tracing::warn!(
                            "[LDAP] gidNumber desconocido para {}: {}, se asigna Student",
                            username,
                            gid
                        );
                        UserRole::Student
                    }
                    None => {
                        tracing::warn!(
                            "[LDAP] usuario sin gidNumber: {}, se asigna Student",
                            username
                        );
                        UserRole::Student
                    }
                };

                Some(LdapUserInfo { email, name, role })
            })
            .ok_or_else(|| {
                tracing::error!(
                    "[!] no se encontró correo electrónico para el usuario: {username}"
                );

                AppError::LdapEmailNotFound
            })?;

        tracing::info!(
            "[✓] Información obtenida para {}: email={}, role={:?}",
            username,
            user_info.email,
            user_info.role
        );

        Ok(user_info)
    }
}
