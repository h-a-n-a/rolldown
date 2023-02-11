use rayon::prelude::*;
use rustc_hash::FxHashMap as HashMap;

use crate::{
  Asset, BundleResult, Chunk, CodeSplitter, FinalizeBundleContext, Graph, InputOptions,
  ModuleRefMutById, OutputOptions, SplitPointIdToChunkId,
};

#[derive(Debug)]
pub struct Bundle<'a> {
  pub input_options: &'a InputOptions,
  pub output_options: &'a OutputOptions,
  pub graph: &'a mut Graph,
  split_point_id_to_chunk_id: SplitPointIdToChunkId,
}

impl<'a> Bundle<'a> {
  pub fn new(
    input_options: &'a InputOptions,
    output_options: &'a OutputOptions,
    graph: &'a mut Graph,
  ) -> Self {
    Self {
      input_options,
      output_options,
      graph,
      split_point_id_to_chunk_id: Default::default(),
    }
  }

  pub fn generate(&mut self) -> BundleResult<Vec<Asset>> {
    let chunks = self.generate_chunks()?;
    let mut chunk_by_id = chunks
      .into_iter()
      .map(|c| (c.id.clone(), c))
      .collect::<HashMap<_, _>>();

    chunk_by_id.values_mut().par_bridge().for_each(|chunk| {
      chunk.gen_file_name(self.output_options);
    });

    let mut module_mut_ref_by_id = self
      .graph
      .module_by_id
      .iter_mut()
      .collect::<HashMap<_, _>>();

    let chunk_filename_by_id = chunk_by_id
      .values()
      .map(|chunk| (chunk.id.clone(), chunk.filename.clone().unwrap()))
      .collect::<HashMap<_, _>>();

    let chunk_and_modules = chunk_by_id
      .values_mut()
      .map(|chunk| {
        let module_mut_ref_by_id: ModuleRefMutById = chunk
          .modules
          .iter()
          .filter(|id| !id.is_external())
          .map(|id| {
            let (key, module) = module_mut_ref_by_id.remove_entry(id).unwrap();
            (key, module)
          })
          .collect();

        (chunk, module_mut_ref_by_id)
      })
      .collect::<Vec<_>>();

    chunk_and_modules
      .into_iter()
      .par_bridge()
      .for_each(|(chunk, module_mut_ref_by_id)| {
        chunk.finalize(FinalizeBundleContext {
          modules: module_mut_ref_by_id,
          uf: &self.graph.uf,
          output_options: self.output_options,
          split_point_id_to_chunk_id: &self.split_point_id_to_chunk_id,
          chunk_filename_by_id: &chunk_filename_by_id,
          unresolved_ctxt: self.graph.unresolved_ctxt,
        });
      });

    let chunks = chunk_by_id
      .values()
      .map(|chunk| {
        let code = chunk.render(
          crate::RenderContext {},
          self.graph,
          self.input_options,
          self.output_options,
        );

        code.map(|code| Asset {
          content: code,
          filename: chunk.filename.clone().unwrap(),
        })
      })
      .try_collect::<Vec<_>>()?;

    Ok(chunks)
  }

  fn generate_chunks(&mut self) -> BundleResult<Vec<Chunk>> {
    let code_splitter =
      CodeSplitter::new(self.graph.entries.clone(), self.graph, self.input_options);
    let chunk_graph = code_splitter.split()?;
    chunk_graph.chunk_by_id.values().for_each(|chunk| {
      chunk.modules.iter().for_each(|module_id| {
        let module = self.graph.module_by_id.get(module_id).unwrap();
        chunk.runtime_helpers.extend_from(module.runtime_helpers());
      });
    });
    self.split_point_id_to_chunk_id = chunk_graph.split_point_to_chunk;

    Ok(chunk_graph.chunk_by_id.into_values().collect())
  }
}
