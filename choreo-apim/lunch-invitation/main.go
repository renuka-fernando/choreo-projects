package main

import (
	"bytes"
	"encoding/json"
	"log/slog"
	"net/http"
	"os"
)

const MESSAGE = `
🥪 *Hey APIM_TEAM, It's Lunch Time!* 🍽️
Thanks for the incredible work you’ve been doing — you’ve more than earned this break! 🎉

Here’s the plan:
1. Step away and take a well-deserved break 🔔
2. Enjoy some lunch and fresh juice or biscuits 💧🍹
3. Return refreshed and recharged to keep being awesome! ⚡️💪
`

func main() {
	slog.Info("Starting lunch invitor...")
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
