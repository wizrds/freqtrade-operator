{{/*
Expand the name of the chart.
*/}}
{{- define "freqtrade-operator.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "freqtrade-operator.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create a fully qualified name for the webhook service.
*/}}
{{- define "freqtrade-operator-webhook.fullname" -}}
{{- printf "%s-webhook" (include "freqtrade-operator.fullname" .) }}
{{- end }}

{{/*
Create a fully qualified name for the controller.
*/}}
{{- define "freqtrade-operator-controller.fullname" -}}
{{- printf "%s-controller" (include "freqtrade-operator.fullname" .) }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "freqtrade-operator.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "freqtrade-operator.labels" -}}
helm.sh/chart: {{ include "freqtrade-operator.chart" . }}
{{ include "freqtrade-operator.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{- define "freqtrade-operator-webhook.labels" -}}
{{ include "freqtrade-operator.labels" . }}
app.kubernetes.io/component: webhook
{{- end }}

{{- define "freqtrade-operator-controller.labels" -}}
{{ include "freqtrade-operator.labels" . }}
app.kubernetes.io/component: controller
{{- end }}

{{/*
Selector labels
*/}}
{{- define "freqtrade-operator.selectorLabels" -}}
app.kubernetes.io/name: {{ include "freqtrade-operator.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "freqtrade-operator.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "freqtrade-operator.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{/*
Generate the TLS certificates
*/}}
{{- define "freqtrade-operator-webhook.tls" -}}
{{- $ca := genCA "freqtrade-operator" 365 -}}
{{- $altNames := list ( printf "%s.%s.svc" (include "freqtrade-operator-webhook.fullname" . ) .Release.Namespace) (printf "%s.%s" (include "freqtrade-operator-webhook.fullname" . ) .Release.Namespace) }}
{{- $cert := genSignedCert (printf "%s-tls" (include "freqtrade-operator-webhook.fullname" . )) nil $altNames 365 $ca -}}
{{- $dict := dict "cert" $cert.Cert "key" $cert.Key "ca" $ca.Cert -}}
{{- $dict | toYaml -}}
{{- end -}}