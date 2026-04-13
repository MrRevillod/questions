#!/usr/bin/env sh
set -eu

KUBECONFIG_PATH="/workspace/config/deployer/kubectl/kubeconfig"

connect_vpn() {
	if [ "${SKIP_VPN:-0}" = "1" ]; then
		return
	fi

	bash /opt/scripts/vpn.sh
}

ensure_k8s_host_resolution() {
	server_url=$(awk '/^[[:space:]]*server:[[:space:]]/ {print $2; exit}' "$KUBECONFIG_PATH")
	if [ -z "${server_url:-}" ]; then
		return
	fi

	server_host=$(printf '%s' "$server_url" | sed -E 's#^https?://##' | cut -d/ -f1 | cut -d: -f1)
	if [ -z "${server_host:-}" ]; then
		return
	fi

	if grep -Eq "[[:space:]]${server_host}([[:space:]]|$)" /etc/hosts; then
		return
	fi

	endpoint=$(printf '%s' "${KUBERNETES_ENDPOINT:-}" | sed -e 's/\r$//' -e ':a;s/^"\(.*\)"$/\1/;ta' -e ":b;s/^'\(.*\)'$/\1/;tb")
	endpoint_ip=$(printf '%s' "$endpoint" | cut -d/ -f1)

	if [ -z "${endpoint_ip:-}" ]; then
		echo "[deploy] cannot resolve ${server_host} and KUBERNETES_ENDPOINT is empty" >&2
		exit 1
	fi

	echo "[deploy] adding host mapping ${server_host} -> ${endpoint_ip}"
	echo "${endpoint_ip} ${server_host}" >> /etc/hosts
}

run_deploy() {
	NAMESPACE="${K8S_NAMESPACE:-ramtun}"
	MANIFESTS_DIR="${K8S_MANIFESTS_DIR:-/workspace/config/deployer/k8s}"

	echo "[deploy] namespace: ${NAMESPACE}"
	echo "[deploy] manifests: ${MANIFESTS_DIR}"

	if ! kubectl get namespace "${NAMESPACE}" >/dev/null 2>&1; then
		echo "[deploy] namespace ${NAMESPACE} not found or no access" >&2
		exit 1
	fi

	perm_missing=0

	while IFS= read -r perm; do
		[ -z "$perm" ] && continue
		verb=$(printf '%s' "$perm" | awk '{print $1}')
		resource=$(printf '%s' "$perm" | awk '{print $2}')
		if ! kubectl auth can-i "$verb" "$resource" -n "$NAMESPACE" >/dev/null 2>&1; then
			echo "[deploy] missing permission: $verb $resource in namespace $NAMESPACE" >&2
			perm_missing=1
		fi
	done <<'EOF'
create deployments.apps
create services
create ingresses.networking.k8s.io
get secrets
create secrets
get clusters.postgresql.cnpg.io
create clusters.postgresql.cnpg.io
EOF

	if [ "$perm_missing" -ne 0 ]; then
		echo "[deploy] insufficient RBAC permissions; aborting before apply" >&2
		exit 1
	fi

	echo "[deploy] applying manifests"
	kubectl apply -R -f "${MANIFESTS_DIR}"

	echo "[deploy] restarting deployments to pick up secret changes"
	kubectl -n "${NAMESPACE}" rollout restart deployment/ramtun-server deployment/ramtun-client

	if ! kubectl -n "${NAMESPACE}" rollout status deployment/ramtun-server --timeout=180s; then
		echo "[deploy] rollout failed for ramtun-server" >&2
		kubectl -n "${NAMESPACE}" get pods -l app=ramtun-server -o wide || true
		kubectl -n "${NAMESPACE}" logs deployment/ramtun-server --tail=120 || true
		exit 1
	fi

	if ! kubectl -n "${NAMESPACE}" rollout status deployment/ramtun-client --timeout=180s; then
		echo "[deploy] rollout failed for ramtun-client" >&2
		kubectl -n "${NAMESPACE}" get pods -l app=ramtun-client -o wide || true
		kubectl -n "${NAMESPACE}" logs deployment/ramtun-client --tail=120 || true
		exit 1
	fi

	echo "[deploy] current resources"
	kubectl -n "${NAMESPACE}" get deploy,svc,ingress,pods
}

connect_vpn
if [ ! -f "$KUBECONFIG_PATH" ]; then
	echo "[deploy] kubeconfig not found at $KUBECONFIG_PATH" >&2
	exit 1
fi

export KUBECONFIG="$KUBECONFIG_PATH"
ensure_k8s_host_resolution

if [ "$#" -eq 0 ] || [ "$1" = "deploy" ]; then
	run_deploy
	exit 0
fi

exec "$@"
