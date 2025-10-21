# version_settings() enforces a minimum Tilt version
# https://docs.tilt.dev/api.html#api.version_settings
version_settings(constraint='>=0.35.0')


IMG = 'freqtrade-operator'

load('ext://restart_process', 'docker_build_with_restart')

k8s_yaml(helm(
    'deploy/helm/freqtrade-operator-crds',
))
k8s_yaml(helm(
    'deploy/helm/freqtrade-operator',
    name='freqtrade-operator',
    namespace='default',
    set=['image.repository=' + IMG],
    skip_crds=True,
))

docker_build(
    IMG,
    context='.',
    dockerfile='build/docker/Dockerfile'
)