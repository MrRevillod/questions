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

	endpoint="${KUBERNETES_ENDPOINT:-}"
	endpoint_ip=$(printf '%s' "$endpoint" | cut -d/ -f1)

	if [ -z "${endpoint_ip:-}" ]; then
		echo "[deploy] cannot resolve ${server_host} and KUBERNETES_ENDPOINT is empty" >&2
		exit 1
	fi

	echo "[deploy] adding host mapping ${server_host} -> ${endpoint_ip}"
	echo "${endpoint_ip} ${server_host}" >> /etc/hosts
}

namespace() {
	printf '%s' "${K8S_NAMESPACE:-ramtun}"
}

manifests_dir() {
	printf '%s' "${K8S_MANIFESTS_DIR:-/workspace/config/deployer/k8s}"
}

ensure_namespace_access() {
	ns=$(namespace)
	if ! kubectl get namespace "$ns" >/dev/null 2>&1; then
		echo "[deploy] namespace $ns not found or no access" >&2
		exit 1
	fi
}

pod_by_label() {
	ns="$1"
	label="$2"
	pod=$(kubectl -n "$ns" get pod -l "$label" -o jsonpath='{.items[0].metadata.name}' 2>/dev/null || true)
	if [ -z "$pod" ]; then
		echo "[deploy] no pod found for label $label in namespace $ns" >&2
		exit 1
	fi
	printf '%s' "$pod"
}

rollout_with_logs() {
	ns="$1"
	deploy_name="$2"
	label="$3"
	if ! kubectl -n "$ns" rollout status "deployment/$deploy_name" --timeout=180s; then
		echo "[deploy] rollout failed for $deploy_name" >&2
		kubectl -n "$ns" get pods -l "$label" -o wide || true
		kubectl -n "$ns" logs "deployment/$deploy_name" --tail=120 || true
		exit 1
	fi
}

cmd_deploy() {
	ns=$(namespace)
	md=$(manifests_dir)

	echo "[deploy] namespace: $ns"
	echo "[deploy] manifests: $md"

	ensure_namespace_access
	kubectl apply -R -f "$md"
	rollout_with_logs "$ns" ramtun-server app=ramtun-server
	rollout_with_logs "$ns" ramtun-client app=ramtun-client
	kubectl -n "$ns" get deploy,svc,ingress,pods
}

cmd_db() {
	ns=$(namespace)
	ensure_namespace_access
	cluster_name="${POSTGRES_CLUSTER_NAME:-postgres-cluster}"
	secret_name="${POSTGRES_SECRET_NAME:-postgres-secret}"
	db_name="${POSTGRES_DB:-ramtun}"
	pod=$(pod_by_label "$ns" "cnpg.io/cluster=$cluster_name")
	user=$(kubectl -n "$ns" get secret "$secret_name" -o jsonpath='{.data.username}' | base64 -d)
	pass=$(kubectl -n "$ns" get secret "$secret_name" -o jsonpath='{.data.password}' | base64 -d)

	if [ -t 0 ] && [ -t 1 ]; then
		kubectl -n "$ns" exec -it "$pod" -- env PGPASSWORD="$pass" psql -h 127.0.0.1 -U "$user" -d "$db_name" "$@"
	else
		kubectl -n "$ns" exec -i "$pod" -- env PGPASSWORD="$pass" psql -h 127.0.0.1 -U "$user" -d "$db_name" "$@"
	fi
}

cmd_usage() {
	cat <<'EOF'
usage: deployer <command>

commands:
  deploy                       apply manifests and rollout app
  db [psql args...]            run psql against postgres-cluster
  sh                           open shell in deployer container
  kubectl <args...>            passthrough kubectl command
EOF
}

connect_vpn
if [ ! -f "$KUBECONFIG_PATH" ]; then
	echo "[deploy] kubeconfig not found at $KUBECONFIG_PATH" >&2
	exit 1
fi
export KUBECONFIG="$KUBECONFIG_PATH"
ensure_k8s_host_resolution

cmd="${1:-deploy}"
[ "$#" -gt 0 ] && shift

case "$cmd" in
	deploy)
		cmd_deploy "$@"
		;;
	sh)
		exec sh "$@"
		;;
	db)
		cmd_db "$@"
		;;
	kubectl|k)
		exec kubectl "$@"
		;;
	help|-h|--help)
		cmd_usage
		;;
	*)
		echo "unknown command: $cmd" >&2
		cmd_usage
		exit 1
		;;
esac
