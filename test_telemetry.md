# Telemetry Verification Summary

To check if data is being sent to Jaeger, open your browser and go to the Jaeger UI. Look for your service name in the filter dropdown and see if any traces appear when you search for them.

To verify if Prometheus is scraping properly, visit the Prometheus Targets page in your browser. Check that the status of the "code-rag-server" job is listed as "UP" and showing a green status.
