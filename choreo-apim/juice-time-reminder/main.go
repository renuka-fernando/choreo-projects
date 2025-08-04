package main

import (
	"bytes"
	"encoding/json"
	"log/slog"
	"net/http"
	"os"
)

const MESSAGE = `Hey CHOREO_APIM Team!
*It’s Juice Time! 🧃🎉 🍹🍹🍹*
`

func main() {
	slog.Info("Starting juice time reminder")
	url := os.Getenv("GCHAT_URL")
	message := os.Getenv("MESSAGE")
	if url == "" {
		slog.Error("The environment variable GCHAT_URL is empty")
		os.Exit(1)
	}
	if message == "" {
		message = MESSAGE
	}

	payload := map[string]string{"text": message}
	jsonPayload, err := json.Marshal(payload)
	if err != nil {
		slog.Error("Error marshaling JSON payload", err)
		return
	}

	resp, err := http.Post(url, "application/json", bytes.NewBuffer(jsonPayload))
	if err != nil {
		slog.Error("Error sending message", err)
		return
	}
	defer resp.Body.Close()

	slog.Info("Message sent successfully", "status", resp.Status)
}
