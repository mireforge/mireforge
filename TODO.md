# Todo

```rust
    let mut item_sort_data: Vec<(usize, i32, PipelineId, MaterialId, TextureId)> =
        self.items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let material = materials.get_weak(item.material_ref).expect("material exists");
                let pipeline_id = material.pipeline_id;
                let texture_id = material.primary_texture_id;

                (index, item.position.z, pipeline_id, item.material_ref, texture_id)
            })
            .collect();

    // Sort by z first (transparency), then pipeline, then material, then texture
    item_sort_data.sort_by(|a, b| {
        .1.cmp(&b.1)
            .then(a.2.cmp(&b.2))
            .then(a.3.cmp(&b.3))
            .then(a.4.cmp(&b.4))
    });
```

## New render

```rust
pub fn render(
    &mut self,
    render_pass: &mut RenderPass,
    materials: &Assets<EnumMaterial>,
    fonts: &Assets<Font>,
    now: Millis,
) {
    trace!("start render()");
    self.last_render_at = now;

    // Viewport calculations and set camera to bind group 0 etc should be same
    // self.prepare_render(materials, fonts);
    // ...


    let mut current_pipeline_id: Option<RenderPipelineId> = None;

    // Batches are sorted by Z, pipeline, material type, material settings (primary texture)
    for &(weak_material_ref, start, count) in &self.batch_offsets {
        let material = materials
            .get_weak(weak_material_ref)
            .expect("no such material");

        // Check if we need to switch the pipeline
        let pipeline_id = material.get_pipeline_id();
        if current_pipeline_id != Some(pipeline_id) {
            render_pass.set_pipeline(material.get_pipeline());
            current_pipeline_id = Some(pipeline_id);
        }

        // Each material should bind its own groups. start at bind group 1
        match material {
            Material::Something() => {

            }
            Material::SomethingElse() => {

            }
        }

        // Issue the instanced draw call for the batch
        trace!(material=%weak_material_ref, start=%start, count=%count, "draw instanced");
        render_pass.draw_indexed(0..num_indices, 0, start..(start + count));
    }

    self.items.clear();
}
```
