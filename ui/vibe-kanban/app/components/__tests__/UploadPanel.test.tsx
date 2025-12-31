import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterAll, beforeAll, describe, expect, it, vi } from "vitest";
import { act } from "react-dom/test-utils";

import type { BoardState } from "../useBoardState";
import { UploadPanel } from "../UploadPanel";

const OriginalFormData = global.FormData;

beforeAll(() => {
  global.FormData = class FormDataMock extends OriginalFormData {
    constructor(form?: HTMLFormElement | undefined) {
      super();
      if (form) {
        const fileInput = form.querySelector<HTMLInputElement>("input[name='file']");
        if (fileInput?.files?.[0]) {
          this.append("file", fileInput.files[0]);
        }
        const dropType = form.querySelector<HTMLSelectElement>("select[name='dropType']");
        if (dropType) {
          this.append("dropType", dropType.value);
        }
      }
    }
  } as unknown as typeof FormData;
});

afterAll(() => {
  global.FormData = OriginalFormData;
});

describe("UploadPanel", () => {
  it("submits selected files with the active drop type", async () => {
    const uploadArtifact = vi.fn().mockResolvedValue(undefined);
    const state = {
      uploadReceipts: [
        {
          id: "receipt-1",
          workspaceId: "studio",
          dropId: "drop-1",
          dropType: "repos",
          originalName: "example.zip",
          casKeys: ["hash-primary", "hash-secondary"],
          receiptPath: "/tmp/receipt.json",
          uploadedAt: new Date().toISOString(),
          uploadedBy: { id: "user-1", name: "Ava" },
        },
      ],
      uploadArtifact,
    } as unknown as BoardState;

    render(<UploadPanel state={state} />);

    const fileInput = screen.getByLabelText(/artifact file/i) as HTMLInputElement;
    const file = new File(["content"], "upload.tar.gz", { type: "application/gzip" });
    await act(async () => {
      await userEvent.upload(fileInput, file);
    });
    expect(fileInput.files?.[0]).toBe(file);

    const select = screen.getByLabelText(/drop type/i);
    await act(async () => {
      await userEvent.selectOptions(select, "mirrors");
    });

    const submit = screen.getByRole("button", { name: /send to crc/i });
    await act(async () => {
      await userEvent.click(submit);
    });

    await waitFor(() => {
      expect(uploadArtifact).toHaveBeenCalledWith({ file, dropType: "mirrors" });
    });
    expect(screen.getByText(/hash-primary/)).toBeTruthy();
  });
});
