// Modal + Confirm primitives, recreated from shell.jsx. Closes on Escape and
// on scrim click.

import React, { useEffect } from "react";
import { Eyebrow } from "./ui";

export function Modal({
  children,
  onClose,
  wide,
}: {
  children: React.ReactNode;
  onClose: () => void;
  wide?: boolean;
}) {
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  return (
    <div
      className="scrim"
      role="presentation"
      onMouseDown={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
    >
      <div className={"modal" + (wide ? " wide" : "")} role="dialog" aria-modal="true">
        {children}
      </div>
    </div>
  );
}

export function Confirm({
  title,
  body,
  confirmLabel = "Confirm",
  danger,
  onConfirm,
  onClose,
}: {
  title: string;
  body: string;
  confirmLabel?: string;
  danger?: boolean;
  onConfirm: () => void;
  onClose: () => void;
}) {
  return (
    <Modal onClose={onClose}>
      <div className="modal-head">
        <Eyebrow label={danger ? "Confirm" : "Please confirm"} />
        <h2>{title}</h2>
      </div>
      <div className="modal-body">
        <p className="page-intro" style={{ margin: 0 }}>
          {body}
        </p>
      </div>
      <div className="modal-foot">
        <button className="btn btn-ghost" onClick={onClose}>
          Cancel
        </button>
        <button
          className={"btn " + (danger ? "btn-danger" : "btn-gold")}
          onClick={() => {
            onConfirm();
            onClose();
          }}
        >
          {confirmLabel}
        </button>
      </div>
    </Modal>
  );
}
