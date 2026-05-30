import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { RoleChip, StatusPill, Segmented, Toggle } from "./ui";

describe("ui components", () => {
  it("RoleChip renders the name and role data attribute", () => {
    const { container } = render(<RoleChip roleKey="owner" name="Owner" />);
    expect(screen.getByText("Owner")).toBeInTheDocument();
    expect(container.querySelector('.rchip[data-role="owner"]')).toBeTruthy();
  });

  it("StatusPill maps status to a label", () => {
    render(<StatusPill status="deactivated" />);
    expect(screen.getByText("Deactivated")).toBeInTheDocument();
  });

  it("Segmented marks the active option and reports changes", async () => {
    const onChange = vi.fn();
    render(
      <Segmented
        value="all"
        onChange={onChange}
        options={[
          { value: "all", label: "All" },
          { value: "active", label: "Active" },
        ]}
      />,
    );
    expect(screen.getByRole("button", { name: "All" })).toHaveClass("on");
    await userEvent.click(screen.getByRole("button", { name: "Active" }));
    expect(onChange).toHaveBeenCalledWith("active");
  });

  it("Toggle reflects and reports state", async () => {
    const onChange = vi.fn();
    render(<Toggle on={false} onChange={onChange} />);
    await userEvent.click(screen.getByRole("checkbox"));
    expect(onChange).toHaveBeenCalledWith(true);
  });
});
