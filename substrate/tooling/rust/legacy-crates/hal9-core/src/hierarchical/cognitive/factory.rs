//! Factory for creating cognitive units

use std::sync::Arc;
use uuid::Uuid;
use crate::Result;
use super::*;

/// Default factory implementation for creating cognitive units
pub struct DefaultCognitiveFactory {
    protocol_manager: Option<Arc<crate::hierarchical::protocol::ProtocolManager>>,
}

impl Default for DefaultCognitiveFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultCognitiveFactory {
    pub fn new() -> Self {
        Self {
            protocol_manager: None,
        }
    }
    
    pub fn with_protocol_manager(mut self, manager: Arc<crate::hierarchical::protocol::ProtocolManager>) -> Self {
        self.protocol_manager = Some(manager);
        self
    }
}

impl CognitiveFactory for DefaultCognitiveFactory {
    fn create_unit(
        &self,
        layer: CognitiveLayer,
        config: CognitiveConfig,
    ) -> Result<Box<dyn CognitiveUnit<Input = CognitiveInput, Output = CognitiveOutput, State = BasicCognitiveState>>> {
        match layer {
            CognitiveLayer::Reflexive => {
                let neuron = Box::new(CognitiveUnitAdapter::new(
                    L1ReflexiveNeuron::new(config)
                ));
                
                // Set protocols if available
                if let Some(manager) = &self.protocol_manager {
                    if let Some(signal_proto) = manager.get_protocol("signal-protocol") {
                        // neuron.inner.set_signal_protocol(signal_proto);
                    }
                }
                
                Ok(neuron)
            }
            
            CognitiveLayer::Implementation => {
                let neuron = Box::new(CognitiveUnitAdapter::new(
                    L2ImplementationNeuron::new(config)
                ));
                
                // Set protocols if available
                if let Some(manager) = &self.protocol_manager {
                    if let Some(gradient_proto) = manager.get_protocol("gradient-protocol") {
                        // neuron.inner.set_gradient_protocol(gradient_proto);
                    }
                }
                
                Ok(neuron)
            }
            
            CognitiveLayer::Operational => {
                Ok(Box::new(CognitiveUnitAdapter::new(
                    L3OperationalNeuron::new(config)
                )))
            }
            
            CognitiveLayer::Tactical => {
                Ok(Box::new(CognitiveUnitAdapter::new(
                    L4TacticalNeuron::new(config)
                )))
            }
            
            CognitiveLayer::Strategic => {
                let neuron = Box::new(CognitiveUnitAdapter::new(
                    L5StrategicNeuron::new(config)
                ));
                
                // Set protocols if available
                if let Some(manager) = &self.protocol_manager {
                    if let Some(consensus_proto) = manager.get_protocol("consensus-protocol") {
                        // neuron.inner.set_consensus_protocol(consensus_proto);
                    }
                }
                
                Ok(neuron)
            }
        }
    }
}

/// Adapter to convert specific neuron states to BasicCognitiveState
struct CognitiveUnitAdapter<N: CognitiveUnit> {
    inner: N,
}

impl<N: CognitiveUnit> CognitiveUnitAdapter<N> {
    fn new(inner: N) -> Self {
        Self { inner }
    }
}

#[async_trait]
impl<N> CognitiveUnit for CognitiveUnitAdapter<N>
where
    N: CognitiveUnit<Input = CognitiveInput, Output = CognitiveOutput>,
    N::State: CognitiveState,
{
    type Input = CognitiveInput;
    type Output = CognitiveOutput;
    type State = BasicCognitiveState;
    
    fn id(&self) -> &Uuid {
        self.inner.id()
    }
    
    fn layer(&self) -> CognitiveLayer {
        self.inner.layer()
    }
    
    async fn process(&mut self, input: Self::Input) -> Result<Self::Output> {
        self.inner.process(input).await
    }
    
    async fn learn(&mut self, gradient: LearningGradient) -> Result<()> {
        self.inner.learn(gradient).await
    }
    
    async fn introspect(&self) -> Self::State {
        let inner_state = self.inner.introspect().await;
        BasicCognitiveState {
            unit_id: *self.inner.id(),
            layer: self.inner.layer(),
            metrics: inner_state.metrics(),
            parameters: HashMap::new(), // Could be extracted from inner state
        }
    }
    
    async fn reset(&mut self) -> Result<()> {
        self.inner.reset().await
    }
}

/// Builder for creating configured cognitive units
pub struct CognitiveUnitBuilder {
    layer: CognitiveLayer,
    id: Option<Uuid>,
    parameters: HashMap<String, f32>,
    connections: ConnectionConfig,
}

impl CognitiveUnitBuilder {
    pub fn new(layer: CognitiveLayer) -> Self {
        Self {
            layer,
            id: None,
            parameters: HashMap::new(),
            connections: ConnectionConfig {
                upward_connections: vec![],
                lateral_connections: vec![],
                downward_connections: vec![],
            },
        }
    }
    
    pub fn with_id(mut self, id: Uuid) -> Self {
        self.id = Some(id);
        self
    }
    
    pub fn with_parameter(mut self, name: &str, value: f32) -> Self {
        self.parameters.insert(name.to_string(), value);
        self
    }
    
    pub fn with_upward_connection(mut self, neuron_id: Uuid) -> Self {
        self.connections.upward_connections.push(neuron_id);
        self
    }
    
    pub fn with_lateral_connection(mut self, neuron_id: Uuid) -> Self {
        self.connections.lateral_connections.push(neuron_id);
        self
    }
    
    pub fn with_downward_connection(mut self, neuron_id: Uuid) -> Self {
        self.connections.downward_connections.push(neuron_id);
        self
    }
    
    pub fn build(self) -> CognitiveConfig {
        CognitiveConfig {
            id: self.id.unwrap_or_else(Uuid::new_v4),
            layer: self.layer,
            initial_parameters: self.parameters,
            connections: self.connections,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_cognitive_factory() {
        let factory = DefaultCognitiveFactory::new();
        
        // Test creating neurons for each layer
        for layer in [
            CognitiveLayer::Reflexive,
            CognitiveLayer::Implementation,
            CognitiveLayer::Operational,
            CognitiveLayer::Tactical,
            CognitiveLayer::Strategic,
        ] {
            let config = CognitiveUnitBuilder::new(layer)
                .with_parameter("learning_rate", 0.01)
                .build();
            
            let mut unit = factory.create_unit(layer, config).unwrap();
            assert_eq!(unit.layer(), layer);
            
            // Test basic processing
            let input = CognitiveInput {
                content: "Test input".to_string(),
                context: HashMap::new(),
                source_layer: None,
            };
            
            let output = unit.process(input).await.unwrap();
            assert!(!output.content.is_empty());
        }
    }
}